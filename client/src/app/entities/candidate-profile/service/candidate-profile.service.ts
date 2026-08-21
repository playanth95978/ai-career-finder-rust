import { HttpClient, HttpResponse, httpResource } from '@angular/common/http';
import { Injectable, computed, inject, signal } from '@angular/core';

import dayjs from 'dayjs/esm';
import { Observable, map } from 'rxjs';

import { ApplicationConfigService } from 'app/core/config/application-config.service';
import { createRequestOption } from 'app/core/request/request-util';
import { isPresent } from 'app/core/util/operators';
import { ICandidateProfile, NewCandidateProfile } from '../candidate-profile.model';

export type PartialUpdateCandidateProfile = Partial<ICandidateProfile> & Pick<ICandidateProfile, 'id'>;

type RestOf<T extends ICandidateProfile | NewCandidateProfile> = Omit<T, 'embeddedAt' | 'createdAt' | 'updatedAt'> & {
  embeddedAt?: string | null;
  createdAt?: string | null;
  updatedAt?: string | null;
};

export type RestCandidateProfile = RestOf<ICandidateProfile>;

export type NewRestCandidateProfile = RestOf<NewCandidateProfile>;

export type PartialUpdateRestCandidateProfile = RestOf<PartialUpdateCandidateProfile>;

@Injectable()
export class CandidateProfilesService {
  readonly candidateProfilesParams = signal<Record<string, string | number | boolean | readonly (string | number | boolean)[]> | undefined>(
    undefined,
  );
  readonly candidateProfilesResource = httpResource<RestCandidateProfile[]>(() => {
    const params = this.candidateProfilesParams();
    if (!params) {
      return undefined;
    }
    return { url: this.resourceUrl, params };
  });
  /**
   * This signal holds the list of candidateProfile that have been fetched. It is updated when the candidateProfilesResource emits a new value.
   * In case of error while fetching the candidateProfiles, the signal is set to an empty array.
   */
  readonly candidateProfiles = computed(() =>
    (this.candidateProfilesResource.hasValue() ? this.candidateProfilesResource.value() : []).map(item =>
      this.convertValueFromServer(item),
    ),
  );
  protected readonly applicationConfigService = inject(ApplicationConfigService);
  protected readonly resourceUrl = this.applicationConfigService.getEndpointFor('api/candidate-profiles');

  protected convertValueFromServer(restCandidateProfile: RestCandidateProfile): ICandidateProfile {
    return {
      ...restCandidateProfile,
      embeddedAt: restCandidateProfile.embeddedAt ? dayjs(restCandidateProfile.embeddedAt) : undefined,
      createdAt: restCandidateProfile.createdAt ? dayjs(restCandidateProfile.createdAt) : undefined,
      updatedAt: restCandidateProfile.updatedAt ? dayjs(restCandidateProfile.updatedAt) : undefined,
    };
  }
}

@Injectable({ providedIn: 'root' })
export class CandidateProfileService extends CandidateProfilesService {
  protected readonly http = inject(HttpClient);

  create(candidateProfile: NewCandidateProfile): Observable<ICandidateProfile> {
    const copy = this.convertValueFromClient(candidateProfile);
    return this.http.post<RestCandidateProfile>(this.resourceUrl, copy).pipe(map(res => this.convertResponseFromServer(res)));
  }

  update(candidateProfile: ICandidateProfile): Observable<ICandidateProfile> {
    const copy = this.convertValueFromClient(candidateProfile);
    return this.http
      .put<RestCandidateProfile>(`${this.resourceUrl}/${encodeURIComponent(this.getCandidateProfileIdentifier(candidateProfile))}`, copy)
      .pipe(map(res => this.convertResponseFromServer(res)));
  }

  partialUpdate(candidateProfile: PartialUpdateCandidateProfile): Observable<ICandidateProfile> {
    const copy = this.convertValueFromClient(candidateProfile);
    return this.http
      .patch<RestCandidateProfile>(`${this.resourceUrl}/${encodeURIComponent(this.getCandidateProfileIdentifier(candidateProfile))}`, copy)
      .pipe(map(res => this.convertResponseFromServer(res)));
  }

  find(id: number): Observable<ICandidateProfile> {
    return this.http
      .get<RestCandidateProfile>(`${this.resourceUrl}/${encodeURIComponent(id)}`)
      .pipe(map(res => this.convertResponseFromServer(res)));
  }

  query(req?: any): Observable<HttpResponse<ICandidateProfile[]>> {
    const options = createRequestOption(req);
    return this.http
      .get<RestCandidateProfile[]>(this.resourceUrl, { params: options, observe: 'response' })
      .pipe(map(res => res.clone({ body: this.convertResponseArrayFromServer(res.body!) })));
  }

  delete(id: number): Observable<undefined> {
    return this.http.delete<undefined>(`${this.resourceUrl}/${encodeURIComponent(id)}`);
  }

  getCandidateProfileIdentifier(candidateProfile: Pick<ICandidateProfile, 'id'>): number {
    return candidateProfile.id;
  }

  compareCandidateProfile(o1: Pick<ICandidateProfile, 'id'> | null, o2: Pick<ICandidateProfile, 'id'> | null): boolean {
    return o1 && o2 ? this.getCandidateProfileIdentifier(o1) === this.getCandidateProfileIdentifier(o2) : o1 === o2;
  }

  addCandidateProfileToCollectionIfMissing<Type extends Pick<ICandidateProfile, 'id'>>(
    candidateProfileCollection: Type[],
    ...candidateProfilesToCheck: (Type | null | undefined)[]
  ): Type[] {
    const candidateProfiles: Type[] = candidateProfilesToCheck.filter(isPresent);
    if (candidateProfiles.length > 0) {
      const candidateProfileCollectionIdentifiers = candidateProfileCollection.map(candidateProfileItem =>
        this.getCandidateProfileIdentifier(candidateProfileItem),
      );
      const candidateProfilesToAdd = candidateProfiles.filter(candidateProfileItem => {
        const candidateProfileIdentifier = this.getCandidateProfileIdentifier(candidateProfileItem);
        if (candidateProfileCollectionIdentifiers.includes(candidateProfileIdentifier)) {
          return false;
        }
        candidateProfileCollectionIdentifiers.push(candidateProfileIdentifier);
        return true;
      });
      return [...candidateProfilesToAdd, ...candidateProfileCollection];
    }
    return candidateProfileCollection;
  }

  protected convertValueFromClient<T extends ICandidateProfile | NewCandidateProfile | PartialUpdateCandidateProfile>(
    candidateProfile: T,
  ): RestOf<T> {
    return {
      ...candidateProfile,
      embeddedAt: candidateProfile.embeddedAt?.toJSON() ?? null,
      createdAt: candidateProfile.createdAt?.toJSON() ?? null,
      updatedAt: candidateProfile.updatedAt?.toJSON() ?? null,
    };
  }

  protected convertResponseFromServer(res: RestCandidateProfile): ICandidateProfile {
    return this.convertValueFromServer(res);
  }

  protected convertResponseArrayFromServer(res: RestCandidateProfile[]): ICandidateProfile[] {
    return res.map(item => this.convertValueFromServer(item));
  }
}
