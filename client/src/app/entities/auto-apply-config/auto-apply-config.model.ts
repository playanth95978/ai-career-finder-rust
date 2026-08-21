import { AutoApplyMode } from 'app/entities/enumerations/auto-apply-mode.model';

export interface IAutoApplyConfig {
  id: number;
  userId?: string | null;
  mode?: keyof typeof AutoApplyMode | null;
  minScore?: number | null;
  maxPerDay?: number | null;
  sources?: string | null;
}

export type NewAutoApplyConfig = Omit<IAutoApplyConfig, 'id'> & { id: null };
