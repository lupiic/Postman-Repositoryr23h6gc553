<div align="center">
  <img src="asset/logo.svg" width="70%"/>
</div>

# Hora

**[[Homepage](http://horasearch.com/)]** **[[Document](https://horasearch.com/doc)]** **[[Examples](https://horasearch.com/doc/example.html)]**

**_Hora Search Everywhere!_**

Hora - это **приблизительный алгоритм поиска ближайшего соседа** ([wiki](https://en.wikipedia.org/wiki/Nearest_neighbor_search)) библиотека. Мы реализуем весь код на `Rust🦀 ` для надежности, высокого уровня абстракции и высокой скорости, сравнимой с `C++`.

Hora, **`「ほら」`** на японском языке, звучит как `[hōlə]` и означает `Вау`,`Ты видишь!`Или`Посмотри на это!`. Название навеяно известной японской песней **`「小さな恋のうた」`**.

# Демо

**👩 Face-Match [[online demo](https://horasearch.com/#Demos)], попробуй!**

<div align="center">
  <img src="asset/demo3.gif" width="100%"/>
</div>

**🍷 Dream wine comments search [[online demo](https://horasearch.com/#Demos)], попробуй!**

<div align="center">
  <img src="asset/demo2.gif" width="100%"/>
</div>

# ключевая особенность

- **Исполнитель** ⚡️

  - **SIMD-Accelerated ([packed_simd](https://github.com/rust-lang/packed_simd))**
  - **Быстрая реализация алгоритма**
  - **Многопоточная конструкция**

- **Поддерживает несколько языков программирования** ☄️

  - `Python`
  - `Javascript`
  - `Java`
  - `Go` (WIP)
  - `Ruby` (WIP)
  - `Swift` (WIP)
  - `R` (WIP)
  - `Julia` (WIP)
  - **Также может использоваться как услуга**

- **Поддерживает несколько индексов** 🚀

  - `Hierarchical Navigable Small World Graph Index (HNSWIndex)` ([details](https://arxiv.org/abs/1603.09320))
  - `Satellite System Graph (SSGIndex)` ([details](https://arxiv.org/abs/1907.06146))
  - `Product Quantization Inverted File(PQIVFIndex)` ([details](https://lear.inrialpes.fr/pubs/2011/JDS11/jegou_searching_with_quantization.pdf))
  - `Random Projection Tree(RPTIndex)` (LSH, WIP)
  - `BruteFo