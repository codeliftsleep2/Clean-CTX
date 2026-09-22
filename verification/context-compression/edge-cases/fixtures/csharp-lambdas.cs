using System;
using Microsoft.AspNetCore.Mvc;

[ApiController]
[Route("api/edge")]
class EdgeController : ControllerBase {
  public enum State { Ready, Busy }

  [HttpGet("text")]
  public string Run(string value) {
    Func<string, string> map = x => Normalize(x);
    return map(value);
  }

  [HttpGet("number")]
  public string Run(int value) {
    Action first = () => Audit(value);
    Action second = () => Notify(value);
    first();
    second();
    return value.ToString();
  }

  private string Normalize(string value) => value.Trim();
}
