from pysrc.pathfinding import Node, QueueItem, search

import sys

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