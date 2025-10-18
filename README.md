A matrix rain terminal effect. Inspired by `cmatrix`.

To compile and run:

`cargo run`

Options:

- `--color green|red|blue|cyan|magenta|white`: Changes the color of the effect. Defaults to "green".
- `--delay <ms>`: The delay between each step, in milliseconds. Defaults to 60ms.
- `--set ascii|jp`: Which character set to use. Defaults to `ascii`.
    - `ascii`: includes lowercase and uppercase latin letters, arabic numerals, and common punctuation.
    - `jp`: includes characters from the hiragana and katana syllabaries.
- `-r`: Reverts the direction of the rain, making it fall upwards.
