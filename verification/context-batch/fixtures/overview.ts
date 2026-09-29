export interface AuditRecord {
  id: string;
  createdAt: Date;
}

export class AuditReader {
  read(record: AuditRecord): string {
    return `${record.id}:${record.createdAt.toISOString()}`;
  }
}
