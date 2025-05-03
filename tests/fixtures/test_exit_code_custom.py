#!/usr/bin/env python3
import sys

def main():
    exit_code = int(sys.argv[1]) if len(sys.argv) > 1 else 42
    print(f'Custom exit code: {exit_code}')
    return exit_code

if __name__ == "__main__":
    sys.exit(main())