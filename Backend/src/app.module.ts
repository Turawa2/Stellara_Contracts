import { Module } from '@nestjs/common';
import { TypeOrmModule } from '@nestjs/typeorm';
import { ConfigModule, ConfigService } from '@nestjs/config';
import { AppController } from './app.controller';
import { AppService } from './app.service';
import { RedisModule } from './redis/redis.module';
import { VoiceModule } from './voice/voice.module';
import { DatabaseModule } from './database/database.module';
import { StellarMonitorModule } from './stellar-monitor/stellar-monitor.module';
import { WorkflowModule } from './workflow/workflow.module';
import { QueueModule } from './queue/queue.module';
import { AuthModule } from './auth/auth.module';
import { Workflow } from './workflow/entities/workflow.entity';
import { WorkflowStep } from './workflow/entities/workflow-step.entity';
import { User } from './auth/entities/user.entity';
import { WalletBinding } from './auth/entities/wallet-binding.entity';
import { LoginNonce } from './auth/entities/login-nonce.entity';
import { RefreshToken } from './auth/entities/refresh-token.entity';
import { ApiToken } from './auth/entities/api-token.entity';

@Module({
  imports: [
    ConfigModule.forRoot({
      isGlobal: true,
    }),
    TypeOrmModule.forRootAsync({
      imports: [ConfigModule],
      useFactory: (configService: ConfigService) => ({
        type: 'postgres',
        host: configService.get('DB_HOST') || 'localhost',
        port: configService.get('DB_PORT') || 5432,
        username: configService.get('DB_USERNAME') || 'postgres',
        password: configService.get('DB_PASSWORD') || 'password',
        database: configService.get('DB_DATABASE') || 'stellara_workflows',
        entities: [
          Workflow,
          WorkflowStep,
          User,
          WalletBinding,
          LoginNonce,
          RefreshToken,
          ApiToken,
        ],
        synchronize: configService.get('NODE_ENV') === 'development',
        logging: configService.get('NODE_ENV') === 'development',
      }),
      inject: [ConfigService],
    }),
    DatabaseModule,
    RedisModule,
    AuthModule,
    VoiceModule,
    StellarMonitorModule,
    WorkflowModule,
    QueueModule,
  ],
  controllers: [AppController],
  providers: [AppService],
})
export class AppModule {}
