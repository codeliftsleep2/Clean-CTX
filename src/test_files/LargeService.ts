// src/test_files/LargeService.ts
// A large, realistic service file for token reduction benchmarking

import { Injectable, Logger, NotFoundException, BadRequestException } from '@nestjs/common';
import { ConfigService } from '@nestjs/config';
import { EventEmitter2 } from '@nestjs/event-emitter';
import { InjectRepository } from '@nestjs/typeorm';
import { Repository, DataSource, QueryRunner, In, Between, Like, MoreThan } from 'typeorm';
import { UserEntity } from './entities/user.entity';
import { OrderEntity } from './entities/order.entity';
import { ProductEntity } from './entities/product.entity';
import { InvoiceEntity } from './entities/invoice.entity';
import { AuditLogEntity } from './entities/audit-log.entity';
import { CreateUserDto } from './dto/create-user.dto';
import { UpdateUserDto } from './dto/update-user.dto';
import { QueryUserDto } from './dto/query-user.dto';
import { PaginatedResult } from './interfaces/paginated-result.interface';
import { UserCreatedEvent } from './events/user-created.event';
import { UserUpdatedEvent } from './events/user-updated.event';
import { UserDeletedEvent } from './events/user-deleted.event';
import { CacheService } from '../cache/cache.service';
import { NotificationService } from '../notification/notification.service';
import { EncryptionService } from '../security/encryption.service';

@Injectable()
export class UserService {
    private readonly logger = new Logger(UserService.name);
    private readonly CACHE_TTL_SECONDS = 3600;
    private readonly MAX_BATCH_SIZE = 100;
    private readonly cachePrefix = 'user:';

    constructor(
        @InjectRepository(UserEntity)
        private readonly userRepository: Repository<UserEntity>,
        @InjectRepository(OrderEntity)
        private readonly orderRepository: Repository<OrderEntity>,
        @InjectRepository(InvoiceEntity)
        private readonly invoiceRepository: Repository<InvoiceEntity>,
        @InjectRepository(AuditLogEntity)
        private readonly auditLogRepository: Repository<AuditLogEntity>,
        private readonly configService: ConfigService,
        private readonly eventEmitter: EventEmitter2,
        private readonly cacheService: CacheService,
        private readonly notificationService: NotificationService,
        private readonly encryptionService: EncryptionService,
        private readonly dataSource: DataSource,
    ) {}

    public async createUser(dto: CreateUserDto): Promise<UserEntity> {
        const queryRunner = this.dataSource.createQueryRunner();
        await queryRunner.connect();
        await queryRunner.startTransaction();

        try {
            this.logger.log(`Creating user with email: ${dto.email}`);
            
            const existingUser = await this.userRepository.findOne({
                where: { email: dto.email }
            });

            if (existingUser) {
                throw new BadRequestException('User with this email already exists');
            }

            const hashedPassword = await this.encryptionService.hashPassword(dto.password);
            
            const user = this.userRepository.create({
                ...dto,
                passwordHash: hashedPassword,
                createdAt: new Date(),
                updatedAt: new Date(),
                isActive: true,
                emailVerified: false,
                loginAttempts: 0,
            });

            const savedUser = await queryRunner.manager.save(user);

            await this.auditLogRepository.save({
                entityType: 'User',
                entityId: savedUser.id,
                action: 'CREATE',
                performedBy: dto.createdBy || 'system',
                timestamp: new Date(),
                changes: JSON.stringify({ created: dto }),
            });

            await queryRunner.commitTransaction();

            this.eventEmitter.emit(
                'user.created',
                new UserCreatedEvent(savedUser.id, savedUser.email, dto.createdBy)
            );

            await this.cacheService.set(
                `${this.cachePrefix}${savedUser.id}`,
                JSON.stringify(savedUser),
                this.CACHE_TTL_SECONDS
            );

            this.logger.log(`User created successfully: ${savedUser.id}`);
            return savedUser;

        } catch (error) {
            await queryRunner.rollbackTransaction();
            this.logger.error(`Failed to create user: ${error.message}`, error.stack);
            throw error;
        } finally {
            await queryRunner.release();
        }
    }

    public async updateUser(id: string, dto: UpdateUserDto): Promise<UserEntity> {
        const queryRunner = this.dataSource.createQueryRunner();
        await queryRunner.connect();
        await queryRunner.startTransaction();

        try {
            const user = await this.userRepository.findOne({ where: { id } });

            if (!user) {
                throw new NotFoundException(`User with id ${id} not found`);
            }

            const changes: Record<string, any> = {};

            if (dto.email && dto.email !== user.email) {
                const existingUser = await this.userRepository.findOne({
                    where: { email: dto.email }
                });
                if (existingUser && existingUser.id !== id) {
                    throw new BadRequestException('Email already in use');
                }
                changes.email = { from: user.email, to: dto.email };
                user.email = dto.email;
                user.emailVerified = false;
            }

            if (dto.firstName) {
                changes.firstName = { from: user.firstName, to: dto.firstName };
                user.firstName = dto.firstName;
            }

            if (dto.lastName) {
                changes.lastName = { from: user.lastName, to: dto.lastName };
                user.lastName = dto.lastName;
            }

            if (dto.phoneNumber) {
                changes.phoneNumber = { from: user.phoneNumber, to: dto.phoneNumber };
                user.phoneNumber = dto.phoneNumber;
            }

            if (dto.role) {
                changes.role = { from: user.role, to: dto.role };
                user.role = dto.role;
            }

            if (dto.password) {
                changes.passwordHash = { from: '***', to: '***' };
                user.passwordHash = await this.encryptionService.hashPassword(dto.password);
            }

            user.updatedAt = new Date();
            const updatedUser = await queryRunner.manager.save(user);

            await this.auditLogRepository.save({
                entityType: 'User',
                entityId: id,
                action: 'UPDATE',
                performedBy: dto.updatedBy || 'system',
                timestamp: new Date(),
                changes: JSON.stringify(changes),
            });

            await queryRunner.commitTransaction();

            this.eventEmitter.emit(
                'user.updated',
                new UserUpdatedEvent(id, changes, dto.updatedBy)
            );

            await this.cacheService.del(`${this.cachePrefix}${id}`);
            
            if (dto.email) {
                await this.cacheService.del(`${this.cachePrefix}email:${dto.email}`);
            }

            this.logger.log(`User updated successfully: ${id}`);
            return updatedUser;

        } catch (error) {
            await queryRunner.rollbackTransaction();
            this.logger.error(`Failed to update user: ${error.message}`, error.stack);
            throw error;
        } finally {
            await queryRunner.release();
        }
    }

    public async deleteUser(id: string, deletedBy?: string): Promise<void> {
        const queryRunner = this.dataSource.createQueryRunner();
        await queryRunner.connect();
        await queryRunner.startTransaction();

        try {
            const user = await this.userRepository.findOne({
                where: { id },
                relations: ['orders', 'invoices']
            });

            if (!user) {
                throw new NotFoundException(`User with id ${id} not found`);
            }

            const orderCount = user.orders?.length || 0;
            if (orderCount > 0) {
                this.logger.warn(`User ${id} has ${orderCount} active orders. Flagging for soft delete.`);
                
                user.isActive = false;
                user.deletedAt = new Date();
                user.deletedBy = deletedBy || 'system';
                await queryRunner.manager.save(user);

                for (const order of user.orders) {
                    await this.orderRepository.update(order.id, {
                        status: 'cancelled',
                        cancelledAt: new Date(),
                        cancelledBy: deletedBy || 'system',
                    });
                }

            } else {
                await queryRunner.manager.remove(user);
                await this.cacheService.del(`${this.cachePrefix}${id}`);
            }

            await this.auditLogRepository.save({
                entityType: 'User',
                entityId: id,
                action: orderCount > 0 ? 'SOFT_DELETE' : 'DELETE',
                performedBy: deletedBy || 'system',
                timestamp: new Date(),
                changes: JSON.stringify({
                    deleted: true,
                    hadOrders: orderCount > 0,
                }),
            });

            await queryRunner.commitTransaction();

            this.eventEmitter.emit(
                'user.deleted',
                new UserDeletedEvent(id, deletedBy || 'system', orderCount > 0)
            );

            this.logger.log(`User ${id} deleted successfully`);

        } catch (error) {
            await queryRunner.rollbackTransaction();
            this.logger.error(`Failed to delete user: ${error.message}`, error.stack);
            throw error;
        } finally {
            await queryRunner.release();
        }
    }

    public async findUsers(query: QueryUserDto): Promise<PaginatedResult<UserEntity>> {
        this.logger.debug(`Querying users with filters: ${JSON.stringify(query)}`);

        const { page = 1, limit = 20, search, role, isActive, startDate, endDate } = query;
        const skip = (page - 1) * limit;

        const where: any = {};

        if (search) {
            where.firstName = Like(`%${search}%`);
        }

        if (role) {
            where.role = role;
        }

        if (isActive !== undefined) {
            where.isActive = isActive;
        }

        if (startDate && endDate) {
            where.createdAt = Between(new Date(startDate), new Date(endDate));
        } else if (startDate) {
            where.createdAt = MoreThan(new Date(startDate));
        }

        const [users, total] = await this.userRepository.findAndCount({
            where,
            skip,
            take: limit,
            order: { createdAt: 'DESC' },
            relations: ['orders'],
        });

        this.logger.debug(`Found ${total} users matching query`);

        return {
            data: users,
            meta: {
                total,
                page,
                limit,
                totalPages: Math.ceil(total / limit),
                hasNextPage: page * limit < total,
                hasPreviousPage: page > 1,
            },
        };
    }

    public async getUserById(id: string): Promise<UserEntity> {
        const cached = await this.cacheService.get(`${this.cachePrefix}${id}`);
        
        if (cached) {
            this.logger.debug(`Cache hit for user ${id}`);
            return JSON.parse(cached);
        }

        this.logger.debug(`Cache miss for user ${id}, querying database`);
        
        const user = await this.userRepository.findOne({
            where: { id },
            relations: ['orders', 'invoices'],
        });

        if (!user) {
            throw new NotFoundException(`User with id ${id} not found`);
        }

        await this.cacheService.set(
            `${this.cachePrefix}${id}`,
            JSON.stringify(user),
            this.CACHE_TTL_SECONDS
        );

        return user;
    }

    public async bulkCreateUsers(dtos: CreateUserDto[]): Promise<UserEntity[]> {
        if (dtos.length > this.MAX_BATCH_SIZE) {
            throw new BadRequestException(
                `Batch size exceeds maximum of ${this.MAX_BATCH_SIZE}`
            );
        }

        const results: UserEntity[] = [];
        const errors: Array<{ index: number; error: string }> = [];

        for (let i = 0; i < dtos.length; i++) {
            try {
                const user = await this.createUser(dtos[i]);
                results.push(user);
            } catch (error) {
                errors.push({ index: i, error: error.message });
                this.logger.error(`Bulk create failed at index ${i}: ${error.message}`);
            }
        }

        if (errors.length > 0) {
            this.logger.warn(
                `Bulk create completed with ${errors.length} errors out of ${dtos.length}`
            );
        }

        return results;
    }

    public async getUserStats(id: string): Promise<Record<string, any>> {
        const user = await this.getUserById(id);

        const [orderCount, totalSpent, invoiceCount, lastLogin] = await Promise.all([
            this.orderRepository.count({ where: { userId: id } }),
            this.orderRepository
                .createQueryBuilder('order')
                .select('SUM(order.totalAmount)', 'total')
                .where('order.userId = :id', { id })
                .getRawOne(),
            this.invoiceRepository.count({ where: { userId: id } }),
            this.auditLogRepository.findOne({
                where: { entityId: id, action: 'LOGIN' },
                order: { timestamp: 'DESC' },
            }),
        ]);

        return {
            userId: id,
            email: user.email,
            orderCount,
            totalSpent: totalSpent?.total || 0,
            invoiceCount,
            lastLoginAt: lastLogin?.timestamp || null,
            createdAt: user.createdAt,
            isActive: user.isActive,
            accountAgeDays: Math.floor(
                (Date.now() - user.createdAt.getTime()) / (1000 * 60 * 60 * 24)
            ),
        };
    }

    public async healthCheck(): Promise<Record<string, any>> {
        const startTime = Date.now();

        try {
            await this.userRepository.query('SELECT 1');
            
            const userCount = await this.userRepository.count();
            const activeUserCount = await this.userRepository.count({
                where: { isActive: true },
            });

            return {
                status: 'healthy',
                database: 'connected',
                cache: await this.cacheService.ping() ? 'connected' : 'disconnected',
                metrics: {
                    totalUsers: userCount,
                    activeUsers: activeUserCount,
                    responseTimeMs: Date.now() - startTime,
                },
                timestamp: new Date().toISOString(),
            };
        } catch (error) {
            this.logger.error(`Health check failed: ${error.message}`);
            return {
                status: 'unhealthy',
                database: 'disconnected',
                error: error.message,
                timestamp: new Date().toISOString(),
            };
        }
    }
}