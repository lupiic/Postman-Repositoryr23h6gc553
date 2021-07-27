<div align="center">
  <img src="asset/logo.svg" width="70%"/>
</div>

# Hora

**[[Homepage](http://horasearch.com/)]** **[[Document](https://horasearch.com/doc)]** **[[Examples](https://horasearch.com/doc/example.html)]**

**_Hora Search Everywhere!_**

Hora est un **algorithme de recherche du voisin le plus proche approximatif** ([wiki](https://en.wikipedia.org/wiki/Nearest_neighbor_search)). Nous implémentons tout le code dans `Rust🦀` pour une fiabilité, une abstraction de haut niveau et des vitesses élevées comparables à `C++`.

Hora, **`「ほら」`** en japonais, sonne comme `[hōlə]`, et signifie `Wow`, `Vous voyez !` ou ` Regardez ça ! `. Le nom est inspiré d'une célèbre chanson japonaise **`「小さな恋のうた」`**.

# Démos

**👩 Face-Match [[online demo](https://horasearch.com/#Demos)], Essaye!**

<div align="center">
  <img src="asset/demo3.gif" width="100%"/>
</div>

**🍷 Recherche de commentaires sur le vin de rêve [[online demo](https://horasearch.com/#Demos)], Essaye!**

<div align="center">
  <img src="asset/demo2.gif" width="100%"/>
</div>

# Principales caractéristiques

- **Performant** ⚡️

  - **SIMD-Accelerated ([packed_simd](https://github.com/rust-lang/packed_simd))**
  - **Implémentation d'algorithme stable**
  - **Multiple threads design**

- **Prend en charge plusieurs langages de programmation Lib** ☄️

  - `Python`
  - `Javascript`
  - `Java`
  - `Go` (WIP)
  - `Ruby` (WIP)
  - `Swift` (WIP)
  - `R` (WIP)
  - `Julia` (WIP)
  - **Peut également être utilisé comme un service**

- **Prend en charge plusieurs index** 🚀

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
  - Pas de dépendances lourdes, telles que `BLAS`

- **Fiabilité** 🔒

  - Le compilateur `Rust` sécurise tout le code
  - Mémoire gérée par `Rust` pour toutes les bibliothèques de langage telles que `Python's`
  - Large couverture de test

- **Prend en charge plusieurs distances** 🧮

  - `Distance du produit de point`
    - ![equation](https://latex.codecogs.com/gif.latex?D%28x%2Cy%29%20%3D%20%5Csum%7B%28x*y%29%7D)
  - `Distance euclidienne`
    - ![equation](https://latex.codecogs.com/gif.latex?D%28x%2Cy%29%20%3D%20%5Csqrt%7B%5Csum%7B%28x-y%29%5E2%7D%7D)
  - `Distance de Manhattan`
    - ![equation](https://latex.codecogs.com/gif.latex?D%28x%2Cy%29%20%3