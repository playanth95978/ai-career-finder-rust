import dayjs from 'dayjs/esm';

import { EmbeddingStatus } from 'app/entities/enumerations/embedding-status.model';
import { JobSource } from 'app/entities/enumerations/job-source.model';

export interface IJobOffer {
  id: number;
  title?: string | null;
  company?: string | null;
  location?: string | null;
  country?: string | null;
  remote?: boolean | null;
  description?: string | null;
  searchText?: string | null;
  skills?: string | null;
  metadata?: string | null;
  rawPayload?: string | null;
  contentHash?: string | null;
  embeddingStatus?: keyof typeof EmbeddingStatus | null;
  embeddingModel?: string | null;
  reindexVersion?: number | null;
  retryCount?: number | null;
  indexingError?: string | null;
  source?: keyof typeof JobSource | null;
  sourceId?: string | null;
  applyUrl?: string | null;
  salaryMin?: number | null;
  salaryMax?: number | null;
  salaryCurrency?: string | null;
  contractType?: string | null;
  experienceLevel?: string | null;
  category?: string | null;
  sourceCategory?: string | null;
  publishedAt?: dayjs.Dayjs | null;
  createdAt?: dayjs.Dayjs | null;
  indexedAt?: dayjs.Dayjs | null;
  updatedAt?: dayjs.Dayjs | null;
  expiresAt?: dayjs.Dayjs | null;
  lastCheckedAt?: dayjs.Dayjs | null;
}

export type NewJobOffer = Omit<IJobOffer, 'id'> & { id: null };
