FROM rabbitmq:3.13
RUN rabbitmq-plugins enable --offline rabbitmq_stream rabbitmq_stream_management rabbitmq_management rabbitmq_management_agent rabbitmq_web_dispatch rabbitmq_prometheus
ENV RABBITMQ_SERVER_ADDITIONAL_ERL_ARGS='-rabbitmq_stream advertised_host rabbit'
