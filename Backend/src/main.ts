import { NestFactory } from '@nestjs/core';
import { AppModule } from './app.module';
import { RedisIoAdapter } from './websocket/redis-io.adapter';
import { DocumentBuilder, SwaggerModule } from '@nestjs/swagger';
import { ValidationPipe } from '@nestjs/common';
import { ThrottleGuard } from './throttle/throttle.guard';
import { NodeSDK } from '@opentelemetry/sdk-node';
import { getNodeAutoInstrumentations } from '@opentelemetry/auto-instrumentations-node';
import { OTLPTraceExporter } from '@opentelemetry/exporter-trace-otlp-http';

// Initialize OpenTelemetry SDK
const traceExporter = new OTLPTraceExporter({
  url: process.env.OTEL_EXPORTER_OTLP_ENDPOINT || 'http://localhost:4318/v1/traces',
});

const sdk = new NodeSDK({
  traceExporter,
  instrumentations: [getNodeAutoInstrumentations()],
});

sdk.start().then(() => {
  console.log('OpenTelemetry initialized');
}).catch((error) => {
  console.error('Error initializing OpenTelemetry', error);
});

async function bootstrap() {
  const app = await NestFactory.create(AppModule);

  // Ensure OpenTelemetry shutdown on app termination
  const shutdown = async () => {
    await sdk.shutdown();
    console.log('OpenTelemetry shut down');
  };
  process.on('SIGTERM', shutdown);
  process.on('SIGINT', shutdown);

  // Enable validation globally
  app.useGlobalPipes(
    new ValidationPipe({
      whitelist: true,
      forbidNonWhitelisted: true,
      transform: true,
    }),
  );

  // Configure Swagger
  const config = new DocumentBuilder()
    .setTitle('Stellara API')
    .setDescription('API for authentication, monitoring Stellar network events, and delivering webhooks')
    .setVersion('1.0')
    .addTag('Authentication')
    .addTag('Stellar Monitor')
    .addBearerAuth()
    .build();
  const document = SwaggerModule.createDocument(app, config);
  SwaggerModule.setup('api/docs', app, document);

  const redisIoAdapter = new RedisIoAdapter(app);
  await redisIoAdapter.connectToRedis();

  app.useWebSocketAdapter(redisIoAdapter);
  app.useGlobalGuards(app.get(ThrottleGuard));


  await app.listen(process.env.PORT ?? 3000);
}
bootstrap();
