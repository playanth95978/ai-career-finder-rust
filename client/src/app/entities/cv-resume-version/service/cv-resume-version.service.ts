import { HttpClient, HttpResponse, httpResource } from '@angular/common/http';
import { Injectable, computed, inject, signal } from '@angular/core';

import dayjs from 'dayjs/esm';
import { Observable, map } from 'rxjs';

import { ApplicationConfigService } from 'app/core/config/application-config.service';
import { createRequestOption } from 'app/core/request/request-util';
import { isPresent } from 'app/core/util/operators';
import { ICvResumeVersion, NewCvResumeVersion } from '../cv-resume-version.model';

export type PartialUpdateCvResumeVersion = Partial<ICvResumeVersion> & Pick<ICvResumeVersion, 'id'>;

type RestOf<T extends ICvResumeVersion | NewCvResumeVersion> = Omit<T, 'createdAt'> & {
  createdAt?: string | null;
};

export type RestCvResumeVersion = RestOf<ICvResumeVersion>;

export type NewRestCvResumeVersion = RestOf<NewCvResumeVersion>;

export type PartialUpdateRestCvResumeVersion = RestOf<PartialUpdateCvResumeVersion>;

@Injectable()
export class CvResumeVersionsService {
  readonly cvResumeVersionsParams = signal<Record<string, string | number | boolean | readonly (string | number | boolean)[]> | undefined>(
    undefined,
  );
  readonly cvResumeVersionsResource = httpResource<RestCvResumeVersion[]>(() => {
    const params = this.cvResumeVersionsParams();
    if (!params) {
      return undefined;
    }
    return { url: this.resourceUrl, params };
  });
  /**
   * This signal holds the list of cvResumeVersion that have been fetched. It is updated when the cvResumeVersionsResource emits a new value.
   * In case of error while fetching the cvResumeVersions, the signal is set to an empty array.
   */
  readonly cvResumeVersions = computed(() =>
    (this.cvResumeVersionsResource.hasValue() ? this.cvResumeVersionsResource.value() : []).map(item => this.convertValueFromServer(item)),
  );
  protected readonly applicationConfigService = inject(ApplicationConfigService);
  protected readonly resourceUrl = this.applicationConfigService.getEndpointFor('api/cv-resume-versions');

  protected convertValueFromServer(restCvResumeVersion: RestCvResumeVersion): ICvResumeVersion {
    return {
      ...restCvResumeVersion,
      createdAt: restCvResumeVersion.createdAt ? dayjs(restCvResumeVersion.createdAt) : undefined,
    };
  }
}

@Injectable({ providedIn: 'root' })
export class CvResumeVersionService extends CvResumeVersionsService {
  protected readonly http = inject(HttpClient);

  create(cvResumeVersion: NewCvResumeVersion): Observable<ICvResumeVersion> {
    const copy = this.convertValueFromClient(cvResumeVersion);
    return this.http.post<RestCvResumeVersion>(this.resourceUrl, copy).pipe(map(res => this.convertResponseFromServer(res)));
  }

  update(cvResumeVersion: ICvResumeVersion): Observable<ICvResumeVersion> {
    const copy = this.convertValueFromClient(cvResumeVersion);
    return this.http
      .put<RestCvResumeVersion>(`${this.resourceUrl}/${encodeURIComponent(this.getCvResumeVersionIdentifier(cvResumeVersion))}`, copy)
      .pipe(map(res => this.convertResponseFromServer(res)));
  }

  partialUpdate(cvResumeVersion: PartialUpdateCvResumeVersion): Observable<ICvResumeVersion> {
    const copy = this.convertValueFromClient(cvResumeVersion);
    return this.http
      .patch<RestCvResumeVersion>(`${this.resourceUrl}/${encodeURIComponent(this.getCvResumeVersionIdentifier(cvResumeVersion))}`, copy)
      .pipe(map(res => this.convertResponseFromServer(res)));
  }

  find(id: number): Observable<ICvResumeVersion> {
    return this.http
      .get<RestCvResumeVersion>(`${this.resourceUrl}/${encodeURIComponent(id)}`)
      .pipe(map(res => this.convertResponseFromServer(res)));
  }

  query(req?: any): Observable<HttpResponse<ICvResumeVersion[]>> {
    const options = createRequestOption(req);
    return this.http
      .get<RestCvResumeVersion[]>(this.resourceUrl, { params: options, observe: 'response' })
      .pipe(map(res => res.clone({ body: this.convertResponseArrayFromServer(res.body!) })));
  }

  delete(id: number): Observable<undefined> {
    return this.http.delete<undefined>(`${this.resourceUrl}/${encodeURIComponent(id)}`);
  }

  getCvResumeVersionIdentifier(cvResumeVersion: Pick<ICvResumeVersion, 'id'>): number {
    return cvResumeVersion.id;
  }

  compareCvResumeVersion(o1: Pick<ICvResumeVersion, 'id'> | null, o2: Pick<ICvResumeVersion, 'id'> | null): boolean {
    return o1 && o2 ? this.getCvResumeVersionIdentifier(o1) === this.getCvResumeVersionIdentifier(o2) : o1 === o2;
  }

  addCvResumeVersionToCollectionIfMissing<Type extends Pick<ICvResumeVersion, 'id'>>(
    cvResumeVersionCollection: Type[],
    ...cvResumeVersionsToCheck: (Type | null | undefined)[]
  ): Type[] {
    const cvResumeVersions: Type[] = cvResumeVersionsToCheck.filter(isPresent);
    if (cvResumeVersions.length > 0) {
      const cvResumeVersionCollectionIdentifiers = cvResumeVersionCollection.map(cvResumeVersionItem =>
        this.getCvResumeVersionIdentifier(cvResumeVersionItem),
      );
      const cvResumeVersionsToAdd = cvResumeVersions.filter(cvResumeVersionItem => {
        const cvResumeVersionIdentifier = this.getCvResumeVersionIdentifier(cvResumeVersionItem);
        if (cvResumeVersionCollectionIdentifiers.includes(cvResumeVersionIdentifier)) {
          return false;
        }
        cvResumeVersionCollectionIdentifiers.push(cvResumeVersionIdentifier);
        return true;
      });
      return [...cvResumeVersionsToAdd, ...cvResumeVersionCollection];
    }
    return cvResumeVersionCollection;
  }

  protected convertValueFromClient<T extends ICvResumeVersion | NewCvResumeVersion | PartialUpdateCvResumeVersion>(
    cvResumeVersion: T,
  ): RestOf<T> {
    return {
      ...cvResumeVersion,
      createdAt: cvResumeVersion.createdAt?.toJSON() ?? null,
    };
  }

  protected convertResponseFromServer(res: RestCvResumeVersion): ICvResumeVersion {
    return this.convertValueFromServer(res);
  }

  protected convertResponseArrayFromServer(res: RestCvResumeVersion[]): ICvResumeVersion[] {
    return res.map(item => this.convertValueFromServer(item));
  }
}
