# SEE editor

> Simple Easy Editor, is a minimal and easy to use command-line text editor!

## Running

You can just test the editor by running

```
cargo run file_name
```

In the parent directory.

## Language support

The editor currently has language support for golang, rust and python. Simple syntax highlighting can be added easily to other languages. This can be done by using the `keyword_formatter.py` tool to construct an array of keywords. The language keywords are stored in the `language_keywords` folder.

### Primary and secondary keywords

If you use some time to read the code, you will probably notice that the keywords are divided to primary and secondary keywords. Currently I recommend using official language keywords like: `if`, `while`, `function` as primary keywords. Secondary keywords are best used for types like `float64`, `i32` or `complex`.

### Adding new supported filetypes

First add the language's keywords in the `language_keywords` folder using the `<language_name>-primary` and `<language_name>-secondary` format. Then run the tool as follows:

```
python3 keyword_formatter.py language_keywords/<>-primary language_keywords/<>-secondary
```

Then using the formatted keywords add the following code to the `src/filetype.rs` into the `Filetype::from()` method file like so:

```rust
if file_name.ends_with(".<language_extension>") {
    return Self {
        name: String::from("<language_name>"),
        highligh_opts: HighlightOptions {
            numbers: true,
            strings: true,
            characters: true,
            comments: true,
            primary_keywords: vec![....],
            secondary_keywords: vec![....],
        },
    };
}
```

There you go, the language syntax support has been added to the text editor! You can also create a pull request for the language support!
