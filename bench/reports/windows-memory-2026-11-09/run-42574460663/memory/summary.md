Windows memory probe: 4 logical processors, 4 server threads, 16378.7 MiB physical, a balloon of 0 MiB; MIN_AVAILABLE_FOR_ACCURATE 8192 MiB.

| Loaded | Stage | Available | Working set | Shipped credit | Plan sees | Accurate model | Credit = working set | Plan sees | Accurate model |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: | ---: | --- |
| accurate | after-health | 5154.3 | 7895.8 | 2883 | 8037.3 | does not fit | 7895.8 | 13050.1 | fits |
| accurate | after-request | 5569.8 | 8108.8 | 2883 | 8452.8 | fits | 8108.8 | 13678.6 | fits |
| accurate | after-idle | 6923.6 | 6685 | 2883 | 9806.6 | fits | 6685 | 13608.6 | fits |
| fast | after-health | 9345.1 | 4318.1 | 2086 | 11431.1 | fits | 4318.1 | 13663.2 | fits |
| fast | after-request | 9044.3 | 4512.8 | 2086 | 11130.3 | fits | 4512.8 | 13557.1 | fits |
| fast | after-idle | 9085.4 | 4512.8 | 2086 | 11171.4 | fits | 4512.8 | 13598.2 | fits |

| Model | Available before load | After request | Taken by the load | Given back 3 s after exit | Shipped credit | Under-credit |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| accurate | 12872.9 | 5569.8 | 7303.1 | 6740.1 | 2883 | 3857.1 |
| fast | 13667.5 | 9044.3 | 4623.2 | 4511.7 | 2086 | 2425.7 |

With the shipped credit, a machine with A MiB available before any model is loaded:
- loads the accurate model for the first document when A >= 8192;
- with the accurate model loaded, plans the fast one for the next document when A < 12612.1;
- with the fast model loaded, plans the accurate one again when A >= 10729.2.
So the models swap before every document for 10729 <= A < 12612.1 MiB.
And for 8192 <= A < 10729.2 MiB the accurate model reads one document and the fast one the rest.
This runner had A = 12872.9 MiB before the first load.
