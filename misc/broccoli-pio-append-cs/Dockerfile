FROM python:3.12

# Set the working directory
WORKDIR /app

# Install the required dependencies
RUN pip install jupyter
RUN pip install pandas
RUN pip install wavedrom
RUN pip install svgwrite
RUN pip install rp2040-pio-emulator@git+https://github.com/NathanY3G/rp2040-pio-emulator
RUN pip install adafruit-circuitpython-pioasm

# Expose the Jupyter notebook port
EXPOSE 8888

# Set the entrypoint command
ENTRYPOINT ["jupyter", "nbconvert", "--to", "html", "--ExecutePreprocessor.timeout=600", "--execute", "broccoli-pio-append-cs.ipynb", "--output", "broccoli-pio-append-cs-result.html", "--debug"]
