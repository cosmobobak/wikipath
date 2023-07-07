# wikipath

wikipath is a CLI tool to play the Wikipedia Speedrun game.
It finds a path between two Wikipedia articles using only links in the articles.
This project has two different implementations, one in Python and one in Rust.
The Rust implementation searches links by optimising the [damerau-levenshtein](https://en.wikipedia.org/wiki/Damerau%E2%80%93Levenshtein_distance) distance between each link and the target link.
The Python implementation searches by optimising the [cosine similarity](https://en.wikipedia.org/wiki/Cosine_similarity) of the [word2vec](https://en.wikipedia.org/wiki/Word2vec) embeddings of each link and the target link.