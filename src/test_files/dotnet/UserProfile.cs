using AutoMapper;

namespace CleanCtx.TestFiles.Dotnet;

/// <summary>
/// Test fixture: AutoMapper profile for .NET Meta-Layer tests.
/// </summary>
public class UserProfile : Profile
{
    public UserProfile()
    {
        // User mappings
        CreateMap<User, UserDto>();
        CreateMap<CreateUserRequest, User>();
        CreateMap<UpdateUserRequest, User>();

        // Order mappings
        CreateMap<Order, OrderDto>();
        
        // Complex mapping with ForMember
        CreateMap<User, UserSummary>()
            .ForMember(dest => dest.FullName, opt => opt.MapFrom(src => $"{src.FirstName} {src.LastName}"))
            .ForMember(dest => dest.Email, opt => opt.MapFrom(src => src.Email));
    }
}

// DTOs
public class UserDto
{
    public int Id { get; set; }
    public string Email { get; set; }
    public string Name { get; set; }
}

public class UserSummary
{
    public string FullName { get; set; }
    public string Email { get; set; }
}

public class CreateUserRequest
{
    public string Email { get; set; }
    public string Name { get; set; }
}

public class UpdateUserRequest
{
    public string Email { get; set; }
    public string Name { get; set; }
}

public class OrderDto
{
    public int Id { get; set; }
    public decimal Total { get; set; }
    public DateTime CreatedAt { get; set; }
}