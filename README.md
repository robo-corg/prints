# Prints

Work in progress (probably don't use this yet) data driven entity templating for entity component systems.

Configure entity blueprints with easy to read and write blueprint files which can be defined in variety of formats such as [RON](https://github.com/ron-rs/ron) or json.

## Example

### Blueprint
```rust
{
    "Name": "corgi",
    "Transform": Transform(
        translation: (1.0, 0.0, 0.0)
    ),
    "Hitpoints": 150.0,
    "Scene": "models/corgi.glb#Scene0",
    "Attacks": [
        FireBreath,
        Scratch,
        Bark
    ],
}
```

### bevy

TODO