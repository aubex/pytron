import os
import sys

UV_CACHE_DIR = os.environ.get("UV_CACHE_DIR", False)
EXAMPLE_RAISE = os.environ.get("EXAMPLE_RAISE", False)
def main():
    print("Hello from pytron-example!")
    if UV_CACHE_DIR:
        print(f"I've recognized {UV_CACHE_DIR =}")
    
    # Print arguments to show what was passed
    print(f"Arguments received: {sys.argv[1:]}")

    if EXAMPLE_RAISE:
        raise RuntimeError("I am raised from within main.py")


if __name__ == "__main__":
    main()
