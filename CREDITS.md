# Credits

piano-action-sim reuses the work below. Each entry says what is reused, where it lives in this repository, where it comes from, and under which terms.

The project itself is released under the MIT License ([`LICENSE`](LICENSE)); the works below keep their own terms. The bench is not deployed yet: once it is, its build carries the notices its libraries' licenses ask for (planned).

## Libraries in the bench's JavaScript bundle

The packages whose code `pnpm build` puts in the bench's bundle (`apps/bench/dist/`), as the bundle's source maps list them.

| Package | Version | Role | License | Source |
|---|---|---|---|---|
| `react`, `react-dom`, `scheduler` | 19.3.0, 19.3.0, 0.28.0 | the page's components and their rendering | MIT | https://react.dev/ |
| `use-sync-external-store` | 1.7.0 | a dependency of `@react-three/fiber` and `@react-three/drei` | MIT | https://github.com/react/react |
| `three` | 0.186.1 | the solid view's 3D rendering | MIT | https://threejs.org/ |
| `@react-three/fiber` | 9.8.1 | the solid view written as React components | MIT | https://github.com/pmndrs/react-three-fiber |
| `@react-three/drei` | 10.7.9 | the solid view's orbit controls | MIT | https://github.com/pmndrs/drei |
| `three-stdlib` | 2.36.1 | a dependency of `@react-three/drei` | MIT | https://github.com/pmndrs/three-stdlib |
| `its-fine`, `react-use-measure`, `suspend-react`, `zustand` | 2.1.1, 2.1.7, 0.1.3, 4.5.7 and 5.0.15 | dependencies of `@react-three/fiber` and `@react-three/drei` | MIT | https://github.com/pmndrs |
| `@babel/runtime` | 7.29.10 | helpers `@react-three/drei`'s compiled code calls | MIT | https://babel.dev/ |

## Crates in the WebAssembly module

The crates linked into `crates/pa-wasm`'s module at run time; the procedural macros they use run only while it compiles.

| Crate | Version | Role | License | Source |
|---|---|---|---|---|
| `libm` | 0.2.16 | every transcendental function of `pa-core`, the same bits natively and in WebAssembly | MIT | https://github.com/rust-lang/compiler-builtins |
| `serde`, `serde_core` | 1.0.229 | the documents `pa-app` sends across the boundary | MIT OR Apache-2.0 | https://github.com/serde-rs/serde |
| `serde_json` | 1.0.152 | the JSON of those documents and of the profiles | MIT OR Apache-2.0 | https://github.com/serde-rs/json |
| `itoa`, `zmij`, `memchr` | 1.0.18, 1.0.23, 2.8.3 | dependencies of `serde_json` | MIT OR Apache-2.0; MIT; Unlicense OR MIT | https://github.com/dtolnay, https://github.com/BurntSushi/memchr |
| `wasm-bindgen`, `wasm-bindgen-shared` | 0.2.129 | the boundary between the module and JavaScript | MIT OR Apache-2.0 | https://github.com/wasm-bindgen/wasm-bindgen |
| `cfg-if`, `once_cell`, `unicode-ident` | 1.0.5, 1.21.4, 1.0.26 | dependencies of `wasm-bindgen` | MIT OR Apache-2.0; MIT OR Apache-2.0; (MIT OR Apache-2.0) AND Unicode-3.0 | https://github.com/rust-lang/cfg-if, https://github.com/matklad/once_cell, https://github.com/dtolnay/unicode-ident |

## Published data

Numbers taken from published works, each recorded with the locator that finds it in its source. The works themselves stay out of the repository: `data/references/` is ignored by git.

- **Hirschkorn's piano action model.** Martin C. Hirschkorn, *Dynamic Model of a Piano Action Mechanism*, MASc thesis, Systems Design Engineering, University of Waterloo, 2004. What is reused: the 235 values of Appendix A, with the laws and the regulation the appendix sends its reader to, in `data/params/hirschkorn-boston-gp178-note52.json`; the key's outline, as coordinates measured on Figure A.1, in `data/params/hirschkorn-key-outline.json`, bound with the appendix into an instrument by `data/profiles/hirschkorn.json`; the outlines of the wippen, its heel and heel cloth, the jack, the repetition lever and the center pin, as coordinates measured on the same figure, in `data/params/hirschkorn-wippen-outline.json`. Terms: the thesis is its author's work under copyright. The repository reproduces none of its pages or figures, only numbers with their table, row and page, short quotations cited where a value needs its wording, and points measured on a figure.
- **Stanwood's touchweight metrology.** David C. Stanwood, *Standard Protocols of the New Touchweight Metrology*, Piano Technicians Journal, February 2000, pp. 20–23: the conventions the protocols run under, in `data/params/touchweight-protocol.json`. David C. Stanwood, *Strike Weight Curve Standards* and *Hammer Weight Curve Standards*, a handout published at stanwoodpiano.com: the curves' table, in `data/params/stanwood-weight-curves.json`. Terms: the author's work under copyright; the repository reproduces the numbers and the conventions with their locators, not the articles.

## Published methods

Methods taken from published works and written anew from their equations: no text, figure or code of the work is in the repository.

- **Weighted blended order-independent transparency.** Morgan McGuire and Louis Bavoil, "Weighted Blended Order-Independent Transparency", *Journal of Computer Graphics Techniques* 2(2):122–141, 2013, http://jcgt.org/published/0002/02/09/. What is reused: the method only, the two images, their blending and the composite that lays them over the opaque parts, implemented from the paper's equations with an equal weight for every face, in `apps/bench/src/transparency.ts`. Terms: the article's page prints "Copyright: © 2013 Morgan McGuire and Louis Bavoil", and the article states that its authors provide it under the Creative Commons CC BY-ND 3.0 license; the repository reproduces none of its text, figures or code.

## Agent guidance

- **gitmoji:** the catalog of `.claude/skills/writing-commits/gitmoji.md`, which lists every gitmoji with its meaning, from version 3.15.0 of the `gitmojis` package (https://github.com/carloscuesta/gitmoji), under the MIT License:

  ```
  MIT License

  Copyright (c) 2016-2022 Carlos Cuesta

  Permission is hereby granted, free of charge, to any person obtaining a copy
  of this software and associated documentation files (the "Software"), to deal
  in the Software without restriction, including without limitation the rights
  to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
  copies of the Software, and to permit persons to whom the Software is
  furnished to do so, subject to the following conditions:

  The above copyright notice and this permission notice shall be included in all
  copies or substantial portions of the Software.

  THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
  IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
  FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
  AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
  LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
  OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
  SOFTWARE.
  ```

## Development tools

Tools that build, check and test the project and ship nothing of their own in the build.

| Tool | Version | License | Source |
|---|---|---|---|
| Biome | 2.5.15 | MIT OR Apache-2.0 | https://biomejs.dev |
| TypeScript | 7.0.2 | Apache-2.0 | https://www.typescriptlang.org/ |
| Turborepo | 2.11.7 | MIT | https://turborepo.dev |
| Vite | 8.3.4 | MIT | https://vite.dev |
| Vitest | 5.0.3 | MIT | https://vitest.dev |
| wasm-pack | 0.15.0 | MIT OR Apache-2.0 | https://github.com/wasm-bindgen/wasm-pack |
| ts-rs | 12.0.1 | MIT | https://github.com/Aleph-Alpha/ts-rs |
