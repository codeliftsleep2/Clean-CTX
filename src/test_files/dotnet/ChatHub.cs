using Microsoft.AspNetCore.SignalR;

namespace CleanCtx.TestFiles.Dotnet;

/// <summary>
/// Test fixture: SignalR hub for .NET Meta-Layer tests.
/// </summary>
public interface IChatClient
{
    Task ReceiveMessage(string user, string message);
    Task UserJoined(string user);
    Task UserLeft(string user);
}

public class ChatHub : Hub<IChatClient>
{
    public async Task SendMessage(string message)
    {
        await Clients.All.ReceiveMessage(Context.User.Identity.Name, message);
    }

    public async Task SendToUser(string userId, string message)
    {
        await Clients.User(userId).ReceiveMessage(Context.User.Identity.Name, message);
    }

    public override async Task OnConnectedAsync()
    {
        await Clients.Others.UserJoined(Context.User.Identity.Name);
        await base.OnConnectedAsync();
    }

    public override async Task OnDisconnectedAsync(Exception exception)
    {
        await Clients.Others.UserLeft(Context.User.Identity.Name);
        await base.OnDisconnectedAsync(exception);
    }
}