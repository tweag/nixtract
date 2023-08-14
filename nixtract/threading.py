from threading import Thread
from typing import Callable, ParamSpec

Param = ParamSpec("Param")


def thread(f: Callable[Param, None]) -> Callable[Param, Thread]:
    def wrapper(*args, **kwargs):
        return Thread(
            target=f,
            args=args,
            kwargs=kwargs,
        )

    return wrapper  # type: ignore
