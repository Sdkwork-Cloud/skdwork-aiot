import type { LogLevel, LoggerConfig } from '../core/types';
export interface Logger {
    debug(message: string, ...args: unknown[]): void;
    info(message: string, ...args: unknown[]): void;
    warn(message: string, ...args: unknown[]): void;
    error(message: string, ...args: unknown[]): void;
    log(level: LogLevel, message: string, ...args: unknown[]): void;
    setLevel(level: LogLevel): void;
}
export declare class ConsoleLogger implements Logger {
    private level;
    private prefix;
    private timestamp;
    private colors;
    constructor(config?: Partial<LoggerConfig>);
    private formatMessage;
    private getColorCode;
    private getResetCode;
    log(level: LogLevel, message: string, ...args: unknown[]): void;
    debug(message: string, ...args: unknown[]): void;
    info(message: string, ...args: unknown[]): void;
    warn(message: string, ...args: unknown[]): void;
    error(message: string, ...args: unknown[]): void;
    setLevel(level: LogLevel): void;
}
export declare const noopLogger: Logger;
export declare function createLogger(config?: Partial<LoggerConfig>): Logger;
//# sourceMappingURL=logger.d.ts.map