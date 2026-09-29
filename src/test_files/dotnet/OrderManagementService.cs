// src/test_files/dotnet/OrderManagementService.cs
//
// Large synthetic ASP.NET Core order-management fixture for token-reduction
// economics. Self-contained: DTOs, EF Core entities/context, a business-logic
// service with constructor injection, and an MVC controller, all in one file.
// Mirrors the tracked TypeScript/Angular economics fixtures in shape and density.

using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading;
using System.Threading.Tasks;
using Microsoft.AspNetCore.Mvc;
using Microsoft.EntityFrameworkCore;
using Microsoft.Extensions.Logging;

namespace CleanCtx.TestFiles.Dotnet.OrderManagement;

public enum OrderStatus
{
    Pending,
    Processing,
    Shipped,
    Delivered,
    Cancelled,
}

public sealed record Address(
    string Line1,
    string Line2,
    string City,
    string State,
    string PostalCode,
    string Country);

public sealed record CreateOrderRequest(
    string CustomerId,
    Address ShippingAddress,
    List<OrderItemRequest> Items,
    string? PromotionCode);

public sealed record OrderItemRequest(string Sku, int Quantity, decimal UnitPrice);

public sealed record UpdateOrderRequest(
    OrderStatus? Status,
    Address? ShippingAddress,
    string? TrackingNumber);

public sealed record OrderItemResponse(string Sku, int Quantity, decimal UnitPrice, decimal LineTotal);

public sealed record OrderResponse(
    Guid Id,
    string CustomerId,
    OrderStatus Status,
    Address ShippingAddress,
    List<OrderItemResponse> Items,
    decimal Subtotal,
    decimal Discount,
    decimal Tax,
    decimal Total,
    DateTime CreatedAt);

public sealed record PagedResult<T>(IReadOnlyList<T> Items, int TotalCount, int Page, int PageSize);

public class Order
{
    public Guid Id { get; set; }
    public string CustomerId { get; set; } = string.Empty;
    public OrderStatus Status { get; set; }
    public Address ShippingAddress { get; set; } = null!;
    public string? TrackingNumber { get; set; }
    public decimal Subtotal { get; set; }
    public decimal Discount { get; set; }
    public decimal Tax { get; set; }
    public decimal Total { get; set; }
    public DateTime CreatedAt { get; set; }
    public DateTime UpdatedAt { get; set; }
    public List<OrderItem> Items { get; set; } = new();
}

public class OrderItem
{
    public Guid Id { get; set; }
    public Guid OrderId { get; set; }
    public Order Order { get; set; } = null!;
    public string Sku { get; set; } = string.Empty;
    public int Quantity { get; set; }
    public decimal UnitPrice { get; set; }
}

public class OrderManagementDbContext : DbContext
{
    public OrderManagementDbContext(DbContextOptions<OrderManagementDbContext> options)
        : base(options)
    {
    }

    public DbSet<Order> Orders { get; set; }
    public DbSet<OrderItem> OrderItems { get; set; }

    protected override void OnModelCreating(ModelBuilder modelBuilder)
    {
        base.OnModelCreating(modelBuilder);

        modelBuilder.Entity<Order>(entity =>
        {
            entity.HasKey(o => o.Id);
            entity.Property(o => o.CustomerId).IsRequired().HasMaxLength(64);
            entity.Property(o => o.Status).HasConversion<string>().IsRequired();
            entity.Property(o => o.Subtotal).HasColumnType("decimal(18,2)");
            entity.Property(o => o.Discount).HasColumnType("decimal(18,2)");
            entity.Property(o => o.Tax).HasColumnType("decimal(18,2)");
            entity.Property(o => o.Total).HasColumnType("decimal(18,2)");
            entity.HasIndex(o => o.CustomerId);
            entity.HasIndex(o => o.Status);
            entity.HasMany(o => o.Items)
                .WithOne(i => i.Order)
                .HasForeignKey(i => i.OrderId)
                .OnDelete(DeleteBehavior.Cascade);
        });

        modelBuilder.Entity<OrderItem>(entity =>
        {
            entity.HasKey(i => i.Id);
            entity.Property(i => i.Sku).IsRequired().HasMaxLength(64);
            entity.Property(i => i.Quantity).IsRequired();
            entity.Property(i => i.UnitPrice).HasColumnType("decimal(18,2)");
            entity.HasIndex(i => i.OrderId);
        });
    }
}

public sealed class OrderValidationException : Exception
{
    public OrderValidationException(string message) : base(message)
    {
    }
}

public interface IOrderService
{
    Task<OrderResponse> CreateOrderAsync(CreateOrderRequest request, CancellationToken cancellationToken = default);
    Task<OrderResponse> GetOrderAsync(Guid id, CancellationToken cancellationToken = default);
    Task<PagedResult<OrderResponse>> GetOrdersAsync(
        string? customerId,
        OrderStatus? status,
        int page,
        int pageSize,
        CancellationToken cancellationToken = default);
    Task<OrderResponse> UpdateOrderAsync(Guid id, UpdateOrderRequest request, CancellationToken cancellationToken = default);
    Task<OrderResponse> ShipOrderAsync(Guid id, string trackingNumber, CancellationToken cancellationToken = default);
    Task<bool> CancelOrderAsync(Guid id, CancellationToken cancellationToken = default);
    Task<bool> DeleteOrderAsync(Guid id, CancellationToken cancellationToken = default);
}

public class OrderService : IOrderService
{
    private readonly OrderManagementDbContext _db;
    private readonly ILogger<OrderService> _logger;
    private readonly IPricingService _pricing;

    private const decimal SalesTaxRate = 0.0825m;
    private const decimal PromoDiscountRate = 0.10m;
    private const int MaxItemsPerOrder = 50;

    public OrderService(
        OrderManagementDbContext db,
        ILogger<OrderService> logger,
        IPricingService pricing)
    {
        _db = db;
        _logger = logger;
        _pricing = pricing;
    }

    public async Task<OrderResponse> CreateOrderAsync(
        CreateOrderRequest request,
        CancellationToken cancellationToken = default)
    {
        ValidateCreateRequest(request);

        var order = new Order
        {
            Id = Guid.NewGuid(),
            CustomerId = request.CustomerId,
            Status = OrderStatus.Pending,
            ShippingAddress = request.ShippingAddress,
            CreatedAt = DateTime.UtcNow,
            UpdatedAt = DateTime.UtcNow,
        };

        foreach (var item in request.Items)
        {
            order.Items.Add(new OrderItem
            {
                Id = Guid.NewGuid(),
                OrderId = order.Id,
                Sku = item.Sku,
                Quantity = item.Quantity,
                UnitPrice = item.UnitPrice,
            });
        }

        ApplyPricing(order, request.PromotionCode);

        using (var transaction = _db.Database.BeginTransaction())
        {
            try
            {
                _db.Orders.Add(order);
                await _db.SaveChangesAsync(cancellationToken);
                transaction.Commit();
            }
            catch (Exception ex)
            {
                transaction.Rollback();
                _logger.LogError(ex, "Failed to create order for customer {CustomerId}", request.CustomerId);
                throw;
            }
        }

        _logger.LogInformation("Created order {OrderId} for customer {CustomerId}", order.Id, request.CustomerId);
        return ToResponse(order);
    }

    public async Task<OrderResponse> GetOrderAsync(
        Guid id,
        CancellationToken cancellationToken = default)
    {
        var order = await _db.Orders
            .Include(o => o.Items)
            .AsNoTracking()
            .SingleOrDefaultAsync(o => o.Id == id, cancellationToken);

        if (order is null)
        {
            throw new KeyNotFoundException($"Order {id} was not found");
        }

        return ToResponse(order);
    }

    public async Task<PagedResult<OrderResponse>> GetOrdersAsync(
        string? customerId,
        OrderStatus? status,
        int page,
        int pageSize,
        CancellationToken cancellationToken = default)
    {
        var query = _db.Orders
            .Include(o => o.Items)
            .AsNoTracking()
            .AsQueryable();

        if (!string.IsNullOrWhiteSpace(customerId))
        {
            query = query.Where(o => o.CustomerId == customerId);
        }

        if (status.HasValue)
        {
            query = query.Where(o => o.Status == status.Value);
        }

        var totalCount = await query.CountAsync(cancellationToken);
        var items = await query
            .OrderByDescending(o => o.CreatedAt)
            .Skip((page - 1) * pageSize)
            .Take(pageSize)
            .Select(o => ToResponse(o))
            .ToListAsync(cancellationToken);

        return new PagedResult<OrderResponse>(items, totalCount, page, pageSize);
    }

    public async Task<OrderResponse> UpdateOrderAsync(
        Guid id,
        UpdateOrderRequest request,
        CancellationToken cancellationToken = default)
    {
        var order = await _db.Orders
            .Include(o => o.Items)
            .SingleOrDefaultAsync(o => o.Id == id, cancellationToken);

        if (order is null)
        {
            throw new KeyNotFoundException($"Order {id} was not found");
        }

        if (order.Status == OrderStatus.Shipped ||
            order.Status == OrderStatus.Delivered ||
            order.Status == OrderStatus.Cancelled)
        {
            throw new OrderValidationException($"Order {id} cannot be modified while {order.Status}");
        }

        if (request.Status.HasValue)
        {
            order.Status = request.Status.Value;
        }

        if (request.ShippingAddress is not null)
        {
            order.ShippingAddress = request.ShippingAddress;
        }

        if (request.TrackingNumber is not null)
        {
            order.TrackingNumber = request.TrackingNumber;
        }

        order.UpdatedAt = DateTime.UtcNow;
        await _db.SaveChangesAsync(cancellationToken);

        _logger.LogInformation("Updated order {OrderId}", id);
        return ToResponse(order);
    }

    public async Task<OrderResponse> ShipOrderAsync(
        Guid id,
        string trackingNumber,
        CancellationToken cancellationToken = default)
    {
        var order = await _db.Orders
            .Include(o => o.Items)
            .SingleOrDefaultAsync(o => o.Id == id, cancellationToken);

        if (order is null)
        {
            throw new KeyNotFoundException($"Order {id} was not found");
        }

        if (order.Status != OrderStatus.Processing)
        {
            throw new OrderValidationException($"Order {id} must be processing before it can ship");
        }

        order.Status = OrderStatus.Shipped;
        order.TrackingNumber = trackingNumber;
        order.UpdatedAt = DateTime.UtcNow;
        await _db.SaveChangesAsync(cancellationToken);

        _logger.LogInformation("Shipped order {OrderId} with tracking {TrackingNumber}", id, trackingNumber);
        return ToResponse(order);
    }

    public async Task<bool> CancelOrderAsync(
        Guid id,
        CancellationToken cancellationToken = default)
    {
        var order = await _db.Orders.SingleOrDefaultAsync(o => o.Id == id, cancellationToken);

        if (order is null)
        {
            throw new KeyNotFoundException($"Order {id} was not found");
        }

        if (order.Status == OrderStatus.Shipped || order.Status == OrderStatus.Delivered)
        {
            return false;
        }

        order.Status = OrderStatus.Cancelled;
        order.UpdatedAt = DateTime.UtcNow;
        await _db.SaveChangesAsync(cancellationToken);
        return true;
    }

    public async Task<bool> DeleteOrderAsync(
        Guid id,
        CancellationToken cancellationToken = default)
    {
        var order = await _db.Orders.SingleOrDefaultAsync(o => o.Id == id, cancellationToken);

        if (order is null)
        {
            return false;
        }

        _db.Orders.Remove(order);
        await _db.SaveChangesAsync(cancellationToken);
        return true;
    }

    private void ValidateCreateRequest(CreateOrderRequest request)
    {
        if (string.IsNullOrWhiteSpace(request.CustomerId))
        {
            throw new OrderValidationException("CustomerId is required");
        }

        if (request.Items is null || request.Items.Count == 0)
        {
            throw new OrderValidationException("An order must contain at least one item");
        }

        if (request.Items.Count > MaxItemsPerOrder)
        {
            throw new OrderValidationException($"An order may not exceed {MaxItemsPerOrder} items");
        }

        foreach (var item in request.Items)
        {
            if (string.IsNullOrWhiteSpace(item.Sku))
            {
                throw new OrderValidationException("Item Sku is required");
            }

            if (item.Quantity <= 0)
            {
                throw new OrderValidationException($"Item {item.Sku} quantity must be positive");
            }

            if (item.UnitPrice < 0)
            {
                throw new OrderValidationException($"Item {item.Sku} unit price may not be negative");
            }
        }
    }

    private decimal CalculateSubtotal(Order order)
    {
        return order.Items.Sum(item => item.Quantity * item.UnitPrice);
    }

    private void ApplyPricing(Order order, string? promotionCode)
    {
        var subtotal = CalculateSubtotal(order);
        var discount = 0m;

        if (!string.IsNullOrWhiteSpace(promotionCode) &&
            string.Equals(promotionCode, "WELCOME10", StringComparison.OrdinalIgnoreCase))
        {
            discount = Math.Round(subtotal * PromoDiscountRate, 2);
        }

        var shipping = _pricing.CalculateShipping(order.ShippingAddress, order.Items.Count);
        var taxable = subtotal - discount + shipping;
        var tax = Math.Round(taxable * SalesTaxRate, 2);

        order.Subtotal = subtotal;
        order.Discount = discount;
        order.Tax = tax;
        order.Total = subtotal - discount + shipping + tax;
    }

    private static OrderResponse ToResponse(Order order)
    {
        return new OrderResponse(
            order.Id,
            order.CustomerId,
            order.Status,
            order.ShippingAddress,
            order.Items
                .Select(item => new OrderItemResponse(
                    item.Sku,
                    item.Quantity,
                    item.UnitPrice,
                    item.Quantity * item.UnitPrice))
                .ToList(),
            order.Subtotal,
            order.Discount,
            order.Tax,
            order.Total,
            order.CreatedAt);
    }
}

public sealed record ShipOrderRequest(string TrackingNumber);

[ApiController]
[Route("api/[controller]")]
public class OrdersController : ControllerBase
{
    private readonly IOrderService _service;
    private readonly ILogger<OrdersController> _logger;

    public OrdersController(IOrderService service, ILogger<OrdersController> logger)
    {
        _service = service;
        _logger = logger;
    }

    [HttpPost]
    public async Task<ActionResult<OrderResponse>> Create(
        [FromBody] CreateOrderRequest request,
        CancellationToken cancellationToken)
    {
        try
        {
            var order = await _service.CreateOrderAsync(request, cancellationToken);
            return CreatedAtAction(nameof(GetById), new { id = order.Id }, order);
        }
        catch (OrderValidationException ex)
        {
            return BadRequest(new { error = ex.Message });
        }
    }

    [HttpGet("{id}")]
    public async Task<ActionResult<OrderResponse>> GetById(Guid id, CancellationToken cancellationToken)
    {
        try
        {
            return Ok(await _service.GetOrderAsync(id, cancellationToken));
        }
        catch (KeyNotFoundException ex)
        {
            return NotFound(new { error = ex.Message });
        }
    }

    [HttpGet]
    public async Task<ActionResult<PagedResult<OrderResponse>>> Get(
        [FromQuery] string? customerId,
        [FromQuery] OrderStatus? status,
        [FromQuery] int page = 1,
        [FromQuery] int pageSize = 25,
        CancellationToken cancellationToken = default)
    {
        var pageNumber = Math.Max(1, page);
        var size = Math.Clamp(pageSize, 1, 100);
        return Ok(await _service.GetOrdersAsync(customerId, status, pageNumber, size, cancellationToken));
    }

    [HttpPut("{id}")]
    public async Task<ActionResult<OrderResponse>> Update(
        Guid id,
        [FromBody] UpdateOrderRequest request,
        CancellationToken cancellationToken)
    {
        try
        {
            return Ok(await _service.UpdateOrderAsync(id, request, cancellationToken));
        }
        catch (KeyNotFoundException ex)
        {
            return NotFound(new { error = ex.Message });
        }
        catch (OrderValidationException ex)
        {
            return BadRequest(new { error = ex.Message });
        }
    }

    [HttpPost("{id}/ship")]
    public async Task<ActionResult<OrderResponse>> Ship(
        Guid id,
        [FromBody] ShipOrderRequest request,
        CancellationToken cancellationToken)
    {
        try
        {
            return Ok(await _service.ShipOrderAsync(id, request.TrackingNumber, cancellationToken));
        }
        catch (KeyNotFoundException ex)
        {
            return NotFound(new { error = ex.Message });
        }
        catch (OrderValidationException ex)
        {
            return BadRequest(new { error = ex.Message });
        }
    }

    [HttpPost("{id}/cancel")]
    public async Task<IActionResult> Cancel(Guid id, CancellationToken cancellationToken)
    {
        try
        {
            var cancelled = await _service.CancelOrderAsync(id, cancellationToken);
            return cancelled ? NoContent() : BadRequest(new { error = "Order cannot be cancelled" });
        }
        catch (KeyNotFoundException ex)
        {
            return NotFound(new { error = ex.Message });
        }
    }

    [HttpDelete("{id}")]
    public async Task<IActionResult> Delete(Guid id, CancellationToken cancellationToken)
    {
        var deleted = await _service.DeleteOrderAsync(id, cancellationToken);
        return deleted ? NoContent() : NotFound();
    }
}
