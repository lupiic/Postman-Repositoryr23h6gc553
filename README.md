<div align="center">
  <img src="asset/logo.svg" width="70%"/>
</div>

<div align="center">
  <h3>  English | <a href="https://github.com/hora-search/hora/blob/main/README_FR.md"> Français </a> | <a href="https://github.com/hora-search/hora/blob/main/README_JP.md"> 日本語 </a> | <a href="https://github.com/hora-search/hora/blob/main/README_KR.md">한국어</a> | <a href="https://github.com/hora-search/hora/blob/main/README_RU.md">Русский</a> | <a href="https://github.com/hora-search/hora/blob/main/README_CN.md">中文</a> </h3>
</div>

# Hora

**[[Homepage](http://horasearch.com/)]** **[[Document](https://horasearch.com/doc)]** **[[Examples](https://horasearch.com/doc/example.html)]**

**_Hora Search Everywhere!_**

Hora is an **approximate nearest neighbor search algorithm** ([wiki](https://en.wikipedia.org/wiki/Nearest_neighbor_search)) library. We implement all code in `Rust🦀` for reliability, high level abstraction and high speeds comparable to `C++`.

Hora, **`「ほら」`** in Japanese, sounds like `[hōlə]`, and means `Wow`, `You see!` or `Look at that!`. The name is inspired by a famous Japanese song **`「小さな恋のうた」`**.

# Demos

**👩 Face-Match [[online demo](https://horasearch.com/#Demos)], have a try!**

<div align="center">
  <img src="asset/demo3.gif" width="100%"/>
</div>

**🍷 Dream wine comments search [[online demo](https://horasearch.com/#Demos)], have a try!**

<div align="center">
  <img src="asset/demo2.gif" width="100%"/>
</div>

# Features

- **Performant** ⚡️

  - **SIMD-Accelerated ([packed_simd](https://github.com/rust-lang/packed_simd))**
  - **Stable algorithm implementation**
  - **Multiple threads design**

- **Supports Multiple Languages** ☄️

  - `Python`
  - `Javascript`
  - `Java`
  - `Go` (WIP)
  - `Ruby` (WIP)
  - `Swift` (WIP)
  - `R` (WIP)
  - `Julia` (WIP)
  - **Can also be used as a service**

- **Supports Multiple Indexes** 🚀

  - `Hierarchical Navigable Small World Graph Index (HNSWIndex)` ([details](https://arxiv.org/abs/1603.09320))
  - `Satellite System Graph (SSGIndex)` ([details](https://arxiv.org/abs/1907.06146))
  - `Product Quantization Inverted File(PQIVFIndex)` ([details](https://lear.inrialpes.fr/pubs/2011/JDS11/jegou_searching_with_quantization.pdf))
  - `Random Projection Tree(RPTIndex)` (LSH, WIP)
  - `BruteForce (BruteForceIndex)` (naive implementation with SIMD)

- **Portable** 💼

  - Supports `WebAssembly`
  - Supports `Windows`, `Linux` and `OS X`
  - Supports `IOS` and `Android` (WIP)
  - Supports `no_std` (WIP, partial)
  - **No** heavy dependencies, such as `BLAS`

- **Reliability** 🔒

  - `Rust` compiler secures all code
  - Memory managed by `Rust` for all language libraries such as `Python's`
  - Broad testing coverage

- **Supports Multiple Distances** 🧮

  - `Dot Product Distance`
    - ![equation](https://latex.codecogs.com/gif.latex?D%28x%2Cy%29%20%3D%20%5Csum%7B%28x*y%29%7D)
  - `Euclidean Distance`
    - ![equation](https://latex.codecogs.com/gif.latex?D%28x%2Cy%29%20%3D%20%5Csqrt%7B%5Csum%7B%28x-y%29%5E2%7D%7D)
  - `Manhattan Distance`
    - ![equation](https://latex.codecogs.com/gif.latex?D%28x%2Cy%29%20%3D%20%5Csum%7B%7C%28x-y%29%7C%7D)
  - `Cosine Similarity`
    - ![equation](https://latex.codecogs.com/gif.latex?D%28x%2Cy%29%20%3D%20%5Cfrac%7Bx%20*y%7D%7B%7C%7Cx%7C%7C*%7C%7Cy%7C%7C%7D)

- **Productive** ⭐
  - Well documented
  - Elegant, simple and easy to learn API

# Installation

**`Rust`**

in `Cargo.toml`

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

**`Building from source`**

```bash
$ git clone https://github.com/hora-search/hora
$ cargo build
```

# Benchmarks

<img src="asset/fashion-mnist-784-euclidean_10_euclidean.png"/>

by `aws t2.medium (CPU: Intel(R) Xeon(R) CPU E5-2686 v4 @ 2.30GHz)` [more information](https://github.com/hora-search/ann-benchmarks)

# Examples

**`Rust` example** [[more info](https://github.com/hora-search/hora/tree/main/examples)]

```Rust
use hora::core::ann_index::ANNIndex;
use rand::{thread_rng, Rng};
use rand_distr::{Distribution, Normal};

pub fn demo() {
    let n = 1000;
    let dimension = 64;

    // make sample points
    let mut samples = Vec::with_capacity(n);
    let normal = Normal::new(0.0, 10.0).unwrap();
    for _i in 0..n {
        let mut sample = Vec::with_capacity(dimension);
        for _j in 0..dimension {
            sample.push(normal.sample(&mut rand::thread_rng()));
        }
        samples.push(sample);
    }

    // init index
    let mut index = hora::index::hnsw_idx::HNSWIndex::<f32, usize>::new(
        dimension,
        &hora::index::hnsw_params::HNSWParams::<f32>::default(),
    );
    for (i, sample) in samples.iter().enumerate().take(n) {
        // add point
        index.add(sample, i).unwrap();
    }
    index.build(hora::core::metrics::Metric::Euclidean).unwrap();

    let mut rng = thread_rng();
    let target: usize = rng.gen_range(0..n);
    // 523 has neighbors: [523, 762, 364, 268, 561, 231, 380, 817, 331, 246]
    println!(
        "{:?} has neighbors: {:?}",
        target,
        index.search(&samples[target], 10) // search for k nearest neighbors
    );
}
```

thank @vaaaaanquish for this complete pure `Rust 🦀` image search [example](https://github.com/vaaaaanquish/rust-ann-search-example), For more information about this example, you can click [Pure Rust な近似最近傍探索ライブラリ hora を用いた画像検索を実装する](https://vaaaaaanquish.hatenablog.com/entry/2021/08/10/065117)

**`Python` example** [[more info](https://github.com/hora-search/horapy)]

```Python
import numpy as np
from horapy import HNSWIndex

dimension = 50
n = 1000

# init index instance
index = HNSWIndex(dimension, "usize")

samples = np.float32(np.random.rand(n, dimension))
for i in range(0, len(samples)):
    # add node
    index.add(np.float32(samples[i]), i)

index.build("euclidean")  # build index

target = np.random.randint(0, n)
# 410 in Hora ANNIndex <HNSWIndexUsize> (dimensi