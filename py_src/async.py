import asyncio

from betfair_data import api



async def main():
    sid = api.login("", "", "")
    print(await sid)

asyncio.run(main())