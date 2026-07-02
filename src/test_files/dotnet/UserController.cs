using Microsoft.AspNetCore.Mvc;
using System.Collections.Generic;

namespace CleanCtx.TestFiles.Dotnet;

/// <summary>
/// Test fixture: ASP.NET Core controller for .NET Meta-Layer tests.
/// </summary>
[ApiController]
[Route("api/[controller]")]
public class UserController : ControllerBase
{
    private readonly IUserService _userService;

    public UserController(IUserService userService)
    {
        _userService = userService;
    }

    /// <summary>
    /// Get all users
    /// </summary>
    [HttpGet]
    public ActionResult<IEnumerable<UserDto>> GetAll()
    {
        return Ok(_userService.GetAllUsers());
    }

    /// <summary>
    /// Get user by ID
    /// </summary>
    [HttpGet("{id}")]
    public ActionResult<UserDto> GetById(int id)
    {
        var user = _userService.GetUserById(id);
        if (user == null)
            return NotFound();
        return Ok(user);
    }

    /// <summary>
    /// Create a new user
    /// </summary>
    [HttpPost]
    public ActionResult<UserDto> Create([FromBody] CreateUserRequest request)
    {
        var user = _userService.CreateUser(request);
        return CreatedAtAction(nameof(GetById), new { id = user.Id }, user);
    }

    /// <summary>
    /// Update an existing user
    /// </summary>
    [HttpPut("{id}")]
    public IActionResult Update(int id, [FromBody] UpdateUserRequest request)
    {
        _userService.UpdateUser(id, request);
        return NoContent();
    }

    /// <summary>
    /// Delete a user
    /// </summary>
    [HttpDelete("{id}")]
    public IActionResult Delete(int id)
    {
        _userService.DeleteUser(id);
        return NoContent();
    }
}