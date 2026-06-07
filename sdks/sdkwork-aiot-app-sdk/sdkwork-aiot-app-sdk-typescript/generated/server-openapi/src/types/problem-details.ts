export interface ProblemDetails {
  type: string;
  title: string;
  status: number;
  detail?: string;
  traceId?: string;
  code?: string;
}
