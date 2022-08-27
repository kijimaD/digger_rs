![image](https://user-images.githubusercontent.com/11595790/187049845-d7276d98-d270-4e05-832c-5964da37d36d.png)
![image](https://user-images.githubusercontent.com/11595790/187049851-9f2661b4-d671-4ea1-94c9-a78d2227a719.png)

## Feature

- Roguelike
- Keyboard Focused
- Symol encounter & Front view battle

## Operation

| Key       | Effect               |
|-----------|----------------------|
| `Enter`   | OK                   |
| `ESC`     | Quit                 |
| `w/a/s/d` | Movement(orthogonal) |
| `q/e/z/x` | Movement(diagonals)  |
| `Space`   | Skip turn            |
| `g`       | Get item             |
| `i`       | Open inventory       |
| `t`       | Drop item            |
| `r`       | Remove equipment     |
| `/`       | Cheat menu           |

## Development

```shell
# development
$ make build
$ make fmt

# release build
$ make docker
root> cd rust; make release
```

## Credit

I expanded on "Roguelike Tutorial - In Rust": (https://bfnightly.bracketproductions.com) code. Thanks great tutorial!
