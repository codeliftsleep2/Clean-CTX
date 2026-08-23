// src/test_files/dotnet/MultiClassFixture.cs
//
// Multi-class C# / .NET regression fixture.
//
// Tests the per-class metadata invariant:
//   A meta-layer may inspect only the exact source span belonging to the
//   type it is enriching. It must never infer ownership from neighboring
//   or whole-file text.
//
// This file contains multiple classes with different .NET framework patterns
// and plain classes interspersed. Each class's markers must be emitted
// ONLY for that class, never for another class.

using Microsoft.AspNetCore.Mvc;
using Microsoft.EntityFrameworkCore;
using Microsoft.AspNetCore.SignalR;
using System.Threading.Tasks;

namespace CleanCtx.TestFiles.Dotnet.MultiClass;

// ════════════════════════════════════════════════════════════════════════
// Class 1: ASP.NET Core Controller with [ApiController] and [Route]
// Expected: Φ marker with ProductsController
// ════════════════════════════════════════════════════════════════════════
[ApiController]
[Route("api/[controller]")]
public class ProductsController : ControllerBase
{
    [HttpGet]
    public IActionResult GetAll()
    {
        return Ok(new[] { "Product1", "Product2" });
    }
}

// ════════════════════════════════════════════════════════════════════════
// Class 2: Plain POCO — NO framework attributes
// Expected: NO Φ markers
// ════════════════════════════════════════════════════════════════════════
public class Product
{
    public int Id { get; set; }
    public string Name { get; set; } = string.Empty;
}

// ════════════════════════════════════════════════════════════════════════
// Class 3: SignalR Hub
// Expected: Φ marker with NotificationHub
// CRITICAL: Must NOT inherit ProductsController's ASP.NET markers
// ════════════════════════════════════════════════════════════════════════
public class NotificationHub : Hub
{
    public async Task SendMessage(string user, string message)
    {
        await Clients.All.SendAsync("ReceiveMessage", user, message);
    }
}

// ════════════════════════════════════════════════════════════════════════
// Class 4: EF Core DbContext
// Expected: Φ marker with InventoryDbContext
// CRITICAL: Must NOT inherit ProductsController's [ApiController] markers
// ════════════════════════════════════════════════════════════════════════
public class InventoryDbContext : DbContext
{
    public DbSet<Product> Products { get; set; } = null!;
}

// ════════════════════════════════════════════════════════════════════════
// Class 5: Plain POCO — NO framework attributes
// Expected: NO Φ markers
// ════════════════════════════════════════════════════════════════════════
public class InventoryItem
{
    public int Id { get; set; }
    public int Quantity { get; set; }
}