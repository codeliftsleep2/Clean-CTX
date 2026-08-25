using Microsoft.EntityFrameworkCore;
using System.Collections.Generic;

namespace CleanCtx.TestFiles.Dotnet;

/// <summary>
/// Test fixture: Entity Framework Core DbContext for .NET Meta-Layer tests.
/// </summary>
public class AppDbContext : DbContext
{
    public DbSet<User> Users { get; set; }
    public DbSet<Order> Orders { get; set; }
    public DbSet<Product> Products { get; set; }

    public AppDbContext(DbContextOptions<AppDbContext> options)
        : base(options)
    {
    }

    protected override void OnModelCreating(ModelBuilder modelBuilder)
    {
        base.OnModelCreating(modelBuilder);

        // User entity configuration
        modelBuilder.Entity<User>(entity =>
        {
            entity.HasKey(u => u.Id);
            entity.Property(u => u.Email).IsRequired();
            entity.HasIndex(u => u.Email).IsUnique();
            entity.HasMany(u => u.Orders)
                .WithOne(o => o.User)
                .HasForeignKey(o => o.UserId);
        });

        // Order entity configuration
        modelBuilder.Entity<Order>(entity =>
        {
            entity.HasKey(o => o.Id);
            entity.Property(o => o.Total).HasColumnType("decimal(18,2)");
        });
    }
}

/// <summary>
/// User entity
/// </summary>
public class User
{
    public int Id { get; set; }
    public string Email { get; set; }
    public string Name { get; set; }
    public List<Order> Orders { get; set; } = new();
}

/// <summary>
/// Order entity
/// </summary>
public class Order
{
    public int Id { get; set; }
    public int UserId { get; set; }
    public User User { get; set; }
    public decimal Total { get; set; }
    public DateTime CreatedAt { get; set; }
}

/// <summary>
/// Product entity
/// </summary>
public class Product
{
    public int Id { get; set; }
    public string Name { get; set; }
    public decimal Price { get; set; }
}