from .my_project import *


__doc__ = my_project.__doc__
if hasattr(my_project, "__all__"):
    __all__ = my_project.__all__
