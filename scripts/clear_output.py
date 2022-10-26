import os
from pathlib import Path


def main():
    for file in Path("../output").iterdir():
        os.remove(file)


if __name__ == "__main__":
    main()
