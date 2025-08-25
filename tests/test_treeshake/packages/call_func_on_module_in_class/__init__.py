from .hello_class import Hello




def main() -> None:
    hello = Hello()
    print(hello.hello_world())

if __name__ == "__main__":
    main()
