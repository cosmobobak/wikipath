import numpy as np
import requests
import sys
from bs4 import BeautifulSoup
from functools import lru_cache
from dataclasses import dataclass

from pysrc.data import embed_sentence, cosine_similarity

PREFIX = "https://en.wikipedia.org/wiki/"


@dataclass
class Node:
    parent: int
    link: str


@dataclass
class QueueItem:
    node_idx: int
    value: float


@lru_cache(maxsize=None)
def crawl(link: str) -> list[str]:
    """
    Crawl a Wikipedia page for links.
    """
    print(f"[#] requesting {link}")
    try:
        response = requests.get(link)
    except requests.exceptions.ConnectionError:
        print(f"Failed to connect to {link}")
        return []
    soup = BeautifulSoup(response.text, "html.parser")
    links = soup.find_all("a")
    # get all the links that start with PREFIX or /wiki/, and that don't contain a colon
    hrefs = [link.get("href") for link in links]
    hrefs = [h for h in hrefs if h]
    hrefs = [h for h in hrefs if h.startswith(PREFIX) or h.startswith("/wiki/")]
    hrefs = [h for h in hrefs if ":" not in h]
    hrefs = [h for h in hrefs if h != link]
    # normalise the links
    hrefs = [h if h.startswith(PREFIX) else PREFIX + h[6:] for h in hrefs]
    return hrefs


def last_part_of_link(link: str) -> str:
    """
    Get the last part of a Wikipedia link (the part after the last slash)
    e.g. https://en.wikipedia.org/wiki/Python_(programming_language) -> Python_(programming_language)
    """
    return link.split("/")[-1]


def search(
    target_link: str,
    all_links: list[Node],
    queue: list[QueueItem],
) -> int:
    """
    Search for a path from the start link to the target link.
    Search is done in a best-first manner, and if a path is found, the index of the target link in the links list is returned.
    """
    target = last_part_of_link(target_link)
    target_embedding = embed_sentence(target)
    if (target_embedding == np.zeros(300)).all():
        print(f"Failed to embed {target}")
        return -1
    seen = set()
    while True:
        if not queue:
            return -1
        # pop off the last link
        idx = queue.pop().node_idx
        link = all_links[idx].link
        # find the number of hops from the start link
        hops = 0
        moving_idx = idx
        while moving_idx != -1:
            moving_idx = all_links[moving_idx].parent
            hops += 1
        # crawl the link
        hrefs = crawl(link)
        # add the links to the list
        for href in hrefs:
            # check if we've seen this link before
            if href in seen:
                continue
            seen.add(href)
            # add the link to the list
            all_links.append(Node(idx, href))
            # add the link to the queue
            embedding = embed_sentence(last_part_of_link(href))
            similarity = cosine_similarity(target_embedding, embedding)
            value = similarity - 0.1 * hops
            queue.append(QueueItem(len(all_links) - 1, value))
            # check if we've found the target
            if href == target_link:
                return len(all_links) - 1

        # sort the queue
        queue.sort(key=lambda x: x.value)


def main():
    # get start and end links from args
    if len(sys.argv) != 3:
        print("Usage: python main.py <start link> <end link>")
        sys.exit(1)
    start_link = sys.argv[1]
    end_link = sys.argv[2]
    # start the search
    all_links = [Node(-1, start_link)]
    queue = [QueueItem(0, 0)]
    idx = search(end_link, all_links, queue)
    if idx == -1:
        print("No path found")
        sys.exit(1)
    # print the path
    print("Path found:")
    path = []
    while idx != -1:
        path.append(all_links[idx].link)
        idx = all_links[idx].parent
    path.reverse()
    max_idx = len(path) - 1
    padding = len(str(max_idx))
    for i, link in enumerate(path):
        print(f"[{i:0{padding}}] {link}")


if __name__ == "__main__":
    main()
