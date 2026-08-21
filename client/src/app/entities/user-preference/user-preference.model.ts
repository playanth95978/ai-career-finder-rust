export interface IUserPreference {
  id: number;
  userId?: string | null;
  remoteOnly?: boolean | null;
  contractType?: string | null;
  salaryMin?: number | null;
  salaryMax?: number | null;
  preferredRoles?: string | null;
  excludedTechnologies?: string | null;
  preferredLocations?: string | null;
}

export type NewUserPreference = Omit<IUserPreference, 'id'> & { id: null };
