import sys

def main():
    primary_word_list = []
    with open(sys.argv[1], "r") as primary_file:
        primary_word_list = primary_file.read().split()

    secondary_word_list = []
    with open(sys.argv[2], "r") as secondary_file:
        secondary_word_list = secondary_file.read().split()

    print("primary_keywords: vec![")
    for word in primary_word_list:
        print(f'\t"{word}".to_string(),')
    print("],")

    print("secondary_keywords: vec![")
    for word in secondary_word_list:
        print(f'\t"{word}".to_string(),')
    print("],")

    print("You can now add the keywords to the src/filetype.rs folder.")

if __name__ == "__main__":
    main()
