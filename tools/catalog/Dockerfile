# Katalog içe aktarma betiği. Bağımlılığı yoktur; imaj yalnızca Node ve betiğin kendisidir.
FROM node:26-slim
WORKDIR /app
COPY catalog.mjs ./
# Çıktı dosyası ve kaynak önbelleği /data altındaki adlandırılmış volume'da durur. Klasör imajda
# `node` kullanıcısına ait oluşturulur ki boş volume ilk bağlandığında aynı sahiplikle gelsin.
RUN mkdir -p /data && chown node:node /data
ENV CATALOG_CACHE_DIR=/data/.cache
USER node
ENTRYPOINT ["node", "catalog.mjs"]
