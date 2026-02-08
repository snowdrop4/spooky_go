from setuptools import setup
from setuptools_rust import Binding, RustExtension  # type: ignore

setup(
    rust_extensions=[
        RustExtension(
            "spooky_go",
            binding=Binding.PyO3,
            debug=False,
            features=["python"],
        )
    ],
    data_files=[("", ["spooky_go.pyi"])],
    zip_safe=False,
)
