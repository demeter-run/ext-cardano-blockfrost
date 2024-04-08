# Stress testing Blockfrost

This code is to run a stress test on out Blockfrost service. It spawns many clients that hit the API.

To run the stress test:

```sh
pip install -r requirements.txt
locust -f locustfile.py
```

Then you should go to your [browser](http://localhost:8089), add your authorized url and perform the test.
