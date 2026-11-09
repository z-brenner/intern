Windows memory probe: 4 logical processors, 4 server threads, 16378.7 MiB physical, a balloon of 2048 MiB; MIN_AVAILABLE_FOR_ACCURATE 8192 MiB.

| Loaded | Stage | Available | Working set | Shipped credit | Plan sees | Accurate model | Credit = working set | Plan sees | Accurate model |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: | ---: | --- |
| accurate | after-health | 3570.6 | 7895.5 | 2883 | 6453.6 | does not fit | 7895.5 | 11466.1 | fits |
| accurate | after-request | 3534.6 | 8107.4 | 2883 | 6417.6 | does not fit | 8107.4 | 11642 | fits |
| accurate | after-idle | 5865.2 | 6331.7 | 2883 | 8748.2 | fits | 6331.7 | 12196.9 | fits |
| fast | after-health | 8092.3 | 4319 | 2086 | 10178.3 | fits | 4319 | 12411.3 | fits |
| fast | after-request | 8233.5 | 4514.1 | 2086 | 10319.5 | fits | 4514.1 | 12747.6 | fits |
| fast | after-idle | 8815.2 | 4445.3 | 2086 | 10901.2 | fits | 4445.3 | 13260.5 | fits |

| Model | Available before load | After request | Taken by the load | Given back 3 s after exit | Shipped credit | Under-credit |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| accurate | 11461 | 3534.6 | 7926.4 | 6333.2 | 2883 | 3450.2 |
| fast | 12162.2 | 8233.5 | 3928.7 | 4528.1 | 2086 | 2442.1 |

With the shipped credit, a machine with A MiB available before any model is loaded:
- loads the accurate model for the first document when A >= 8192;
- with the accurate model loaded, plans the fast one for the next document when A < 13235.4;
- with the fast model loaded, plans the accurate one again when A >= 10034.7.
So the models swap before every document for 10035 <= A < 13235.4 MiB.
And for 8192 <= A < 10034.7 MiB the accurate model reads one document and the fast one the rest.
This runner had A = 11461 MiB before the first load.
