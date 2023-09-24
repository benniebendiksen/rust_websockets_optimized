import arbitrage_processing
from arbitrage_processing import Client
import asyncio
import time
import itertools
import random

times = 0
n = 3

# url = "wss://testnet.binance.vision/ws-api/v3"
#url = "wss://ws-api.binance.com/ws-api/v3"
url = "http://localhost:50000"
# stream_url = "wss://stream.binance.com:443/"
stream_url = "http://localhost:50000/stream"
api = "0668Tj19rdXW6e8fsiTr0mWidcEYDozuqvQrgNSa6yYjHH4vhTc605UXfjTj5guW"
secret_key = "j8geX4SFisFUBAdBXpywzwNT3wgITMJHePJ7G0UPgz4uLX7IP0DfcIbWjk4QpRxG"
#api = "SLrOc5Rkkgvk6UEradGu9A9ljVpUEk5uO1yxGYDIaMHD27QBnFs6utM1p1VgASXy"
#secret_key = "U8HhVpHUTq4guboJeAlzUmCGyPwi3gJEkzLZM53b9OI0a2VIbstVHwCxH9u031oG"



client = Client(url, stream_url, api, secret_key, thread_num=3)

time.sleep(1)

client.subscribe([("BTC", "USDT", "ADA")])
client.subscribe([("ETH", "BTC", "DOGE")])

async def print_updates():
  asyncio.create_task(add_subscription())

  while True:
    update = await client.get_result()
    print(f"py received order response: {update.symbol} with status: {update.status}")

async def add_subscription():
  await asyncio.sleep(2)
  client.subscribe([("ETH", "USDT", "ADA")])

asyncio.run(print_updates())

# from datetime import datetime
# timestamp = round(datetime.now().timestamp() * 1e3)
# prepared_order = {
#     "apiKey": api,
#     "newOrderRespType": "ACK",
#     "symbol": "BTCUSDT",
#     "side": "SELL",
#     "quantity": "0.0005",
#     "type": "MARKET",
#     "timestamp": timestamp
# }

# arbitrage_processing.prepare_order(json.dumps(prepared_order))
# arbitrage_processing.flush()
# response = arbitrage_processing.receive()
# print(response)

# for a in range(1, n+1):
#     start = time.time()
#     arbitrage_processing.execute_orders('LUNAUSDT', 'SELL', 'MARKET')
#     end = time.time() - start
#     print(f"Round {a} time is: {end}")
#     times += end

# arbitrage_processing.disconnect()

# print(times/n)
