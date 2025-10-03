# 16puzzle
A simple task to get started with Jules
15 Puzzle in Rust with WebAssembly and wgpu

This project implements a classic 15 Puzzle (also known as the sliding puzzle) using Rust, WebAssembly (WASM), and the wgpu graphics API. The puzzle is rendered in a web browser, allowing users to interact with the tiles by clicking on them to slide them into the empty space.
# Features

    Interactive 15 Puzzle: Click on a tile adjacent to the empty space to slide it into the gap.
    Move Counter: Displays the number of moves made so far.
    Restart Button: Resets the puzzle to a new, solvable configuration.
    Solvable Puzzles: Ensures all generated puzzles are mathematically solvable using a proper scrambling method.
    Web-based: Runs in any modern web browser via WASM.

# Requirements

To develop and run this project, you need the following:
1. Rust Toolchain

Install Rust via rustup:
bash

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

Ensure you have:

    rustc (Rust compiler)
    cargo (Rust package manager)

2. WebAssembly Target

Add the WASM target to your Rust toolchain:
bash

rustup target add wasm32-unknown-unknown

3. wasm-bindgen

Used to bridge Rust and JavaScript for WASM:
bash

cargo add wasm-bindgen

4. wasm-bindgen-futures

For async/await support in WASM:
bash

cargo add wasm-bindgen-futures

5. wgpu

The modern GPU abstraction library for rendering:
bash

cargo add wgpu

6. wasm-pack

Tool to build and package Rust projects for the web:
bash

cargo install wasm-pack

7. Python 3 and     python -m http.server

For serving the web app and managing frontend dependencies:

    Install python3

# Project Setup

    Create a new Rust project:
    bash

cargo new --lib 15-puzzle-wasm
cd 15-puzzle-wasm

Add the required dependencies to Cargo.toml:
toml

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
wasm-bindgen = "0.2"
wasm-bindgen-futures = "0.4"
wgpu = "0.16"
rand = "0.8"

Build the project for WASM:
bash

wasm-pack build --target web

Set up the frontend:
Create an index.html file in the project root:
html

<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0"/>
    <title>15 Puzzle</title>
    <style>
        body {
            display: flex;
            justify-content: center;
            align-items: center;
            height: 100vh;
            margin: 0;
            background-color: #f0f0f0;
            font-family: sans-serif;
        }
        .container {
            display: flex;
            flex-direction: column;
            align-items: center;
            gap: 20px;
        }
        .puzzle {
            display: grid;
            grid-template-columns: repeat(4, 60px);
            grid-template-rows: repeat(4, 60px);
            gap: 2px;
        }
        .tile {
            width: 60px;
            height: 60px;
            background-color: #007acc;
            color: white;
            display: flex;
            justify-content: center;
            align-items: center;
            font-size: 24px;
            cursor: pointer;
            border-radius: 4px;
            transition: background-color 0.2s;
        }
        .tile.empty {
            background-color: #ddd;
            cursor: default;
        }
        .tile:hover {
            background-color: #005fa3;
        }
        .controls {
            display: flex;
            gap: 10px;
            justify-content: center;
        }
        button {
            padding: 8px 16px;
            font-size: 16px;
            background-color: #007acc;
            color: white;
            border: none;
            border-radius: 4px;
            cursor: pointer;
        }
        button:hover {
            background-color: #005fa3;
        }
        .counter {
            font-size: 18px;
            font-weight: bold;
            color: #333;
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="counter">Moves: <span id="move-count">0</span></div>
        <div class="puzzle" id="puzzle"></div>
        <div class="controls">
            <button id="restart">Restart</button>
        </div>
    </div>

    <script type="module">
        import init from './pkg/15_puzzle_wasm.js';

        async function main() {
            await init();

            const puzzle = document.getElementById('puzzle');
            const moveCountEl = document.getElementById('move-count');
            const restartBtn = document.getElementById('restart');

            let moves = 0;

            // Initialize the puzzle
            const puzzleInstance = new window.Puzzle();
            puzzleInstance.render(puzzle, moveCountEl, updateMoveCount);
            restartBtn.onclick = () => {
                puzzleInstance.restart();
                moves = 0;
                updateMoveCount(moves);
            };

            function updateMoveCount(count) {
                moves = count;
                moveCountEl.textContent = moves;
            }
        }

        main().catch(console.error);
    </script>
</body>
</html>

Serve the app using a simple HTTP server:
bash

    python -m http.server
    Then open http://localhost:8000 in your browser.

# Puzzle Logic (Solvable Generation)

To ensure every generated puzzle is solvable, the algorithm works as follows:

    Start from the solved state:
    text

    1  2  3  4
    5  6  7  8
    9 10 11 12

13 14 15 _
text


2. Randomly select a valid move (slide a tile into the empty space).

3. Repeat for **between 50 and 500 random steps**.

4. The resulting configuration is guaranteed to be solvable because:
- All valid moves preserve the **parity invariant** of the puzzle.
- Since we start from a solvable state and only apply legal moves, the result remains solvable.

> This method avoids the complexity of checking solvability via inversion count.

---

##  Rendering with wgpu

- The puzzle grid is rendered using `wgpu` for high-performance, hardware-accelerated graphics.
- Each tile is drawn as a colored rectangle with text.
- Click events are captured via JavaScript and forwarded to Rust using `wasm-bindgen`.

---

##  Build & Run

```bash
# Build for WASM
wasm-pack build --target web

# Serve the app
# TODO: figure out the python command line to serve this

Open http://localhost:8000 in your browser.
# Notes

    Performance: wgpu is used for efficient rendering; avoid unnecessary re-renders.
    Error Handling: Use console.error() and wasm-bindgen's #[wasm_bindgen] macros to handle JS-Rust communication safely.
    Extensibility: Add features like timer, difficulty levels, or animation effects in the future.

# License

This project is licensed under the GNU GPL 3.0 License – see LICENSE for details.
# Contributions

Contributions are welcome! Please open an issue or submit a pull request.  No copyright assignment needed, but 
a developer certificate of origin is welcome.ooooooooooo

# Enjoy the 15 Puzzle! Slide those tiles and challenge your logic.

