# Pytron Example

This is an example Python application for the `pytron` tool.

## Usage

This example demonstrates how to use the `pytron` tool to:

1. Zip Python scripts with dependencies
2. Run Python scripts directly or from a zip archive
3. Pass arguments to both `uv run` and to the Python script

## Example Commands

### Zipping files

```
pytron zip -o example.zip
```

### Running scripts

Run a script directly:
```
pytron run main.py
```

Run a script with verbose uv output:
```
pytron run -v main.py
```

Run a script with arguments:
```
pytron run main.py arg1 arg2
```

Run a script with arguments that start with dashes:
```
pytron run main.py --custom-flag
```

Run a script from a zip file:
```
pytron run example.zip main.py
```

Pass arguments to both uv and the script:
```
pytron run -v example.zip --script-arg1 --script-arg2
```

Using a double-dash separator with default zip/script:
```
pytron run -v -- --script-arg
```

## Argument Handling

Arguments are separated in two ways:

1. Using the double-dash (`--`) separator:
   ```
   pytron run [UV_FLAGS] -- [SCRIPT_ARGS]
   ```

2. Using the script/zipfile path as a separator:
   ```
   pytron run [UV_FLAGS] script.py [SCRIPT_ARGS]
   ```

This allows passing arguments that start with hyphens to the Python script without confusion.