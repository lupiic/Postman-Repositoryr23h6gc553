
<div align="center">
  <img src="asset/logo.svg" width="70%"/>
</div>

# Hora

**[[Homepage](http://horasearch.com/)]** **[[Document](https://horasearch.com/doc)]** **[[Examples](https://horasearch.com/doc/example.html)]**

**_Hora Search Everywhere!_**

Horaは**近似最近傍探索アルゴリズムライブラリ** [[Wikipedia](https://ja.wikipedia.org/wiki/%E6%9C%80%E8%BF%91%E5%82%8D%E6%8E%A2%E7%B4%A2)]です。 信頼性、高レベルの抽象化、および `C++`に匹敵する高速性を達成するために、すべてのコードを`Rust🦀`で実装しています。

日本語で「ほら」は、`[hōlə]`のように聞こえます。この名前は、日本の歌「小さな恋のうた」の有名な歌詞「ほら あなたにとって大事な人ほど すぐそばにいるの」にちなんで付けられました。

# デモ

**👩 Face-Match [[online demo](https://horasearch.com/#Demos)]**

<div align="center">
  <img src="asset/demo3.gif" width="100%"/>
</div>

**🍷 Dream wine comments search [[online demo](https://horasearch.com/#Demos)]**

<div align="center">
  <img src="asset/demo2.gif" width="100%"/>
</div>

# 特徴

- **性能** ⚡️

  - **SIMD アクセラレーション ([packed_simd](https://github.com/rust-lang/packed_simd))**
  - **安定したアルゴリズムの実装**
  - **マルチスレッドデザイン**

- **複数のプログラミング言語をサポート** ☄️

  - `Python`
  - `Javascript`
  - `Java`
  - `Go` (WIP)
  - `Ruby` (WIP)
  - `Swift` (WIP)
  - `R` (WIP)
  - `Julia` (WIP)
  - **サービスとしても使用可能**

- **複数のインデックスをサポート** 🚀

  - `Hierarchical Navigable Small World Graph Index (HNSWIndex)` ([details](https://arxiv.org/abs/1603.09320))
  - `Satellite System Graph (SSGIndex)` ([details](https://arxiv.org/abs/1907.06146))
  - `Product Quantization Inverted File(PQIVFIndex)` ([details](https://lear.inrialpes.fr/pubs/2011/JDS11/jegou_searching_with_quantization.pdf))
  - `Random Projection Tree(RPTIndex)` (LSH, WIP)
  - `BruteForce (BruteForceIndex)` (SIMDを使った素朴な実装)

- **移植性** 💼

  - `WebAssembly`対応
  - `Windows`、`Linux`および`OS X`に対応
  - `iOS`および`Android`対応 (WIP)
  - `no_std`対応 (WIP, partial)
  - `BLAS`などの大きな依存関係は**ありません**

- **信頼性** 🔒

  - `Rust`コンパイラはすべてのコードを保護します
  - `Python`などの全ての言語向けのライブラリで`Rust`によるメモリ管理
  - 幅広いテスト範囲

- **複数の距離をサポート** 🧮

  - `Dot Product Distance`
    - ![equation](https://latex.codecogs.com/gif.latex?D%28x%2Cy%29%20%3D%20%5Csum%7B%28x*y%29%7D)
  - `Euclidean Distance`
    - ![equation](https://latex.codecogs.com/gif.latex?D%28x%2Cy%29%20%3D%20%5Csqrt%7B%5Csum%7B%28x-y%29%5E2%7D%7D)
  - `Manhattan Distance`
    - ![equation](https://latex.codecogs.com/gif.latex?D%28x%2Cy%29%20%3D%20%5Csum%7B%7C%28x-y%29%7C%7D)
  - `Cosine Similarity`
    - ![equation](https://latex.codecogs.com/gif.latex?D%28x%2Cy%29%20%3D%20%5Cfrac%7Bx%20*y%7D%7B%7C%7Cx%7C%7C*%7C%7Cy%7C%7C%7D)

- **生産性** ⭐
  - 整備されたドキュメント
  - エレガントかつシンプル、そして習得しやすいAPI

# インストール

`Cargo.toml`で

```toml
[dependencies]
hora = "0.1.1"
```

**`Python`**

```Bash
$ pip install horapy
```

**`Javascript (WebAssembly)`**

```Bash
$ npm i horajs
```

**`ソースコードからビルド`**

```bash
$ git clone https://github.com/hora-search/hora
$ cargo build
```

# ベンチマーク

<img src="asset/fashion-mnist-784-euclidean_10_euclidean.png"/>

by `aws t2.medium (CPU: Intel(R) Xeon(R) CPU E5-2686 v4 @ 2.30GHz)` [more information](https://github.com/hora-search/ann-benchmarks)

# Examples

**`Rust`** [[詳細](https://github.com/hora-search/hora/tree/main/examples)]

```Rust
use hora::core::ann_index::ANNIndex;
use rand::{thread_rng, Rng};
use rand_distr::{Distribution, Normal};

pub fn demo() {
    let n = 1000;
    let dimension = 64;
