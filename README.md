# Harbor 🐍

Harbor is a **Python-like programming language** that compiles to zero-dependency Node.js code. It looks and feels like Python — but with even easier syntax.

## Why Harbor?

- **Python-like syntax** — If you know Python, you already know Harbor
- **Easier than Python** — No `self` in method params, no `new` keyword, no `async/await` boilerplate, `print` works without parens
- **Built-in web server** — Define HTTP APIs in just a few lines
- **Zero runtime dependencies** — Compiles to clean, vanilla Node.js
- **F-strings, classes, modules** — Everything you'd expect from a modern language

## Quick Example

```python
# Variables
name = "Harbor"
version = 2.0

# F-strings
print f"Welcome to {name} v{version}!"

# Functions
def greet(who):
    print f"Hello, {who}!"

greet("World")

# Lists & loops
fruits = ["apple", "banana", "cherry"]
for fruit in fruits:
    print fruit

for i in range(5):
    print i

# If / elif / else
score = 85
if score >= 90:
    print "A"
elif score >= 80:
    print "B"
else:
    print "C"

# Logical operators
if score > 70 and score < 90:
    print "Good job!"

# Membership test
if "banana" in fruits:
    print "We have bananas!"

# Classes — no 'new' keyword, no 'self' in params!
class Dog:
    def init(name, breed):
        self.name = name
        self.breed = breed

    def speak():
        print f"{self.name} says Woof!"

rex = Dog("Rex", "Lab")
rex.speak()

# Error handling
try:
    data = fs.read("file.txt")
except e:
    print f"Error: {e}"

# Built-in web server
server 3000:
    get "/":
        respond {"message": "Hello!"}

    post "/data":
        respond 201 {"received": req.body}
```

## Easier Than Python

| Feature | Python | Harbor |
|---------|--------|--------|
| Print | `print("hello")` | `print "hello"` |
| Class methods | `def speak(self):` | `def speak():` |
| Constructor | `def __init__(self, name):` | `def init(name):` |
| Instantiation | `rex = Dog("Rex")` | `rex = Dog("Rex")` |
| Async functions | `async def fetch():` / `await` | `def fetch():` (auto-async) |
| HTTP server | Flask/Django boilerplate | `server 3000:` |
| Comments | `# comment` | `# comment` |
| F-strings | `f"Hello {name}"` | `f"Hello {name}"` |

## Built-in Functions

Harbor comes with Python-like builtins out of the box:

| Function | Description |
|----------|-------------|
| `print` | Print values (no parens needed!) |
| `len(x)` | Length of string, list, or object |
| `range(n)` | Generate number sequence |
| `str(x)`, `int(x)`, `float(x)` | Type conversion |
| `type(x)` | Get type of value |
| `input(prompt)` | Read user input |
| `abs(x)`, `round(x)` | Math functions |
| `min(...)`, `max(...)` | Min/max values |
| `sum(list)` | Sum of list |
| `sorted(list)` | Sort a list |
| `reversed(list)` | Reverse a list |
| `enumerate(list)` | Index-value pairs |
| `keys(obj)`, `values(obj)`, `items(obj)` | Object helpers |
| `any(list)`, `all(list)` | Boolean checks |
| `chr(n)`, `ord(c)` | Character conversion |

## Operators

```python
# Arithmetic
x = 2 ** 10      # Power: 1024
y = 17 // 5      # Floor division: 3
z = 17 % 5       # Modulo: 2

# Compound assignment
count = 0
count += 1
count -= 1
count *= 2
count /= 2

# Logical (Python-style)
if x > 0 and y > 0:
    print "both positive"

if not done:
    print "still going"

if a or b:
    print "at least one"

# Membership
if "hello" in words:
    print "found it"

if "goodbye" not in words:
    print "not found"
```

## How to Use

### 1. Compile a Harbor file
```bash
cargo run -- main.hb -o output.js
```

### 2. Run directly
```bash
cargo run -- main.hb
```

### 3. Start a web server
```bash
cargo run -- server.hb -o server.js && node server.js
```

## Modules

```python
# utils.hb
export def add(a, b):
    return a + b

export def greet(name):
    print f"Hello, {name}!"
```

```python
# main.hb
import "./utils.hb" as utils
utils.greet("Harbor")

# Or import specific names
from "./utils.hb" import add, greet
print add(5, 3)
```

## Installation

### One-liner (recommended)

```bash
# curl
curl -sSL https://harbor.fluxlinux.xyz/install.sh | bash

# or wget
wget -qO- https://harbor.fluxlinux.xyz/install.sh | bash
```

### From source

```bash
git clone https://github.com/stormyy00/harbor.git
cd harbor
bash install.sh
```

### Options

```bash
# Install a specific version
HARBOR_VERSION=2.0.0 curl -sSL https://harbor.fluxlinux.xyz/install.sh | bash

# Custom install directory
HARBOR_INSTALL_DIR=/usr/local/bin curl -sSL https://harbor.fluxlinux.xyz/install.sh | bash

# Uninstall
curl -sSL https://harbor.fluxlinux.xyz/install.sh | bash -s -- --uninstall
```

## VS Code Extension

The `harbor-vscode` folder contains a VS Code extension with:
- Syntax highlighting with Python-like theme
- Code snippets for all language constructs
- File icon support for `.hb` files

## License

MIT
