FROM python:3.12-slim

ADD ./ /tw

WORKDIR /tw

RUN pip install --no-cache-dir  -r requirements.txt

CMD ["python", "-OO", "-u", "/tw/main.py"]
