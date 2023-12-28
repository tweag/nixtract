def maybe_popleft(deque):
    """
    Pop an item from a deque on the left, returning None if the deque is empty.
    """
    try:
        return deque.popleft()
    except IndexError:
        return None
