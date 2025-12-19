# Async Programming

loft supports asynchronous programming with async/await.

## Async Functions

Declare async functions with the `async` keyword:

```loft
async fn fetch_data() -> str {
    return "data from server";
}
```

## Await

Call async functions with `await`:

```loft
async fn main() {
    let data = await fetch_data();
    term.println(data);
}
```

## Practical Example

```loft
async fn fetch_url(url: str) -> Result {
    // Simulate async operation
    return Result.Ok("response data");
}

async fn process_urls() {
    let urls = [
        "https://api.example.com/1",
        "https://api.example.com/2",
    ];
    
    for url in urls {
        let result = await fetch_url(url);
        match result {
            Result.Ok(data) => term.println(data),
            Result.Err(msg) => term.println(msg),
        };
    }
}
```

## Error Handling

Combine async with error propagation:

```loft
async fn load_config() -> Result {
    let content = await fs.read_async("config.json")?;
    return Result.Ok(content);
}
```
