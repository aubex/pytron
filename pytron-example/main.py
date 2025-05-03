import os
import sys

UV_CACHE_DIR = os.environ.get("UV_CACHE_DIR", False)
def main():
    print("Hello from pytron-example!")
    if UV_CACHE_DIR:
        print(f"I've recognized {UV_CACHE_DIR =}")
    
    # Print arguments to show what was passed
    print(f"Arguments received: {sys.argv[1:]}")


if __name__ == "__main__":
    main()
