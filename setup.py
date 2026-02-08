from setuptools import setup
from setuptools_rust import Binding, RustExtension  # type: ignore

setup(
    rust_extensions=[
        RustExtension(
            "rust_go",
            binding=Binding.PyO3,
            debug=False,
            features=["python"],
        )
    ],
    data_files=[("", ["rust_go.pyi"])],
    zip_safe=False,
)
