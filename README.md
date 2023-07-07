# wikipath

wikipath is a CLI tool to play the Wikipedia Speedrun game.
It finds a path between two Wikipedia articles using only links in the articles.
This project has two different implementations, one in Python and one in Rust.
The Rust implementation searches links by optimising the [damerau-levenshtein](https://en.wikipedia.org/wiki/Damerau%E2%80%93Levenshtein_distance) distance between each link and the target link.
The Python implementation searches by optimising the [cosine similarity](https://en.wikipedia.org/wiki/Cosine_similarity) of the [word2vec](https://en.wikipedia.org/wiki/Word2vec) embeddings of each link and the target link.

example usage:
Rust:
```
$ cargo build --release
$ ./target/release/wikipath https://en.wikipedia.org/wiki/Python_(programming_language) https://en.wikipedia.org/wiki/Computer_programming
```

Python:
```
$ python3 -m venv venv
$ source venv/bin/activate
$ pip install -r requirements.txt
$ python3 pysrc/main.py https://en.wikipedia.org/wiki/Python_(programming_language) https://en.wikipedia.org/wiki/Computer_programming
```

Example output:
```
$ python .\pysrc\main.py https://en.wikipedia.org/wiki/Albert_Einstein https://en.wikipedia.org/wiki/Cat
[#] requesting https://en.wikipedia.org/wiki/Albert_Einstein
[#] requesting https://en.wikipedia.org/wiki/Horace_Tabberer_Brown
[#] requesting https://en.wikipedia.org/wiki/Staffordshire
[#] requesting https://en.wikipedia.org/wiki/Tamworth_(pig)
[#] requesting https://en.wikipedia.org/wiki/Pig
[#] requesting https://en.wikipedia.org/wiki/Animal
Path found:
[0] https://en.wikipedia.org/wiki/Albert_Einstein
[1] https://en.wikipedia.org/wiki/Horace_Tabberer_Brown
[2] https://en.wikipedia.org/wiki/Staffordshire
[3] https://en.wikipedia.org/wiki/Tamworth_(pig)
[4] https://en.wikipedia.org/wiki/Pig
[5] https://en.wikipedia.org/wiki/Animal
[6] https://en.wikipedia.org/wiki/Cat
```