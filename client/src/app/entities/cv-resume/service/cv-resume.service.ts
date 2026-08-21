import { HttpClient, HttpResponse, httpResource } from '@angular/common/http';
import { Injectable, computed, inject, signal } from '@angular/core';

import dayjs from 'dayjs/esm';
import { Observable, map } from 'rxjs';

import { ApplicationConfigService } from 'app/core/config/application-config.service';
import { createRequestOption } from 'app/core/request/request-util';
import { isPresent } from 'app/core/util/operators';
import { ICvResume, NewCvResume } from '../cv-resume.model';

export type PartialUpdateCvResume = Partial<ICvResume> & Pick<ICvResume, 'id'>;

type RestOf<T extends ICvResume | NewCvResume> = Omit<T, 'createdAt' | 'updatedAt'> & {
  createdAt?: string | null;
  updatedAt?: string | null;
};

export type RestCvResume = RestOf<ICvResume>;

export type NewRestCvResume = RestOf<NewCvResume>;

export type PartialUpdateRestCvResume = RestOf<PartialUpdateCvResume>;

@Injectable()
export class CvResumesService {
  readonly cvResumesParams = signal<Record<string, string | number | boolean | readonly (string | number | boolean)[]> | undefined>(
    undefined,
  );
  readonly cvResumesResource = httpResource<RestCvResume[]>(() => {
    const params = this.cvResumesParams();
    if (!params) {
      return undefined;
    }
    return { url: this.resourceUrl, params };
  });
  /**
   * This signal holds the list of cvResume that have been fetched. It is updated when the cvResumesResource emits a new value.
   * In case of error while fetching the cvResumes, the signal is set to an empty array.
   */
  readonly cvResumes = computed(() =>
    (this.cvResumesResource.hasValue() ? this.cvResumesResource.value() : []).map(item => this.convertValueFromServer(item)),
  );
  protected readonly applicationConfigService = inject(ApplicationConfigService);
  protected readonly resourceUrl = this.applicationConfigService.getEndpointFor('api/cv-resumes');

  protected convertValueFromServer(restCvResume: RestCvResume): ICvResume {
    return {
      ...restCvResume,
      createdAt: restCvResume.createdAt ? dayjs(restCvResume.createdAt) : undefined,
      updatedAt: restCvResume.updatedAt ? dayjs(restCvResume.updatedAt) : undefined,
    };
  }
}

@Injectable({ providedIn: 'root' })
export class CvResumeService extends CvResumesService {
  protected readonly http = inject(HttpClient);

  create(cvResume: NewCvResume): Observable<ICvResume> {
    const copy = this.convertValueFromClient(cvResume);
    return this.http.post<RestCvResume>(this.resourceUrl, copy).pipe(map(res => this.convertResponseFromServer(res)));
  }

  update(cvResume: ICvResume): Observable<ICvResume> {
    const copy = this.convertValueFromClient(cvResume);
    return this.http
      .put<RestCvResume>(`${this.resourceUrl}/${encodeURIComponent(this.getCvResumeIdentifier(cvResume))}`, copy)
      .pipe(map(res => this.convertResponseFromServer(res)));
  }

  partialUpdate(cvResume: PartialUpdateCvResume): Observable<ICvResume> {
    const copy = this.convertValueFromClient(cvResume);
    return this.http
      .patch<RestCvResume>(`${this.resourceUrl}/${encodeURIComponent(this.getCvResumeIdentifier(cvResume))}`, copy)
      .pipe(map(res => this.convertResponseFromServer(res)));
  }

  find(id: number): Observable<ICvResume> {
    return this.http
      .get<RestCvResume>(`${this.resourceUrl}/${encodeURIComponent(id)}`)
      .pipe(map(res => this.convertResponseFromServer(res)));
  }

  query(req?: any): Observable<HttpResponse<ICvResume[]>> {
    const options = createRequestOption(req);
    return this.http
      .get<RestCvResume[]>(this.resourceUrl, { params: options, observe: 'response' })
      .pipe(map(res => res.clone({ body: this.convertResponseArrayFromServer(res.body!) })));
  }

  delete(id: number): Observable<undefined> {
    return this.http.delete<undefined>(`${this.resourceUrl}/${encodeURIComponent(id)}`);
  }

  getCvResumeIdentifier(cvResume: Pick<ICvResume, 'id'>): number {
    return cvResume.id;
  }

  compareCvResume(o1: Pick<ICvResume, 'id'> | null, o2: Pick<ICvResume, 'id'> | null): boolean {
    return o1 && o2 ? this.getCvResumeIdentifier(o1) === this.getCvResumeIdentifier(o2) : o1 === o2;
  }

  addCvResumeToCollectionIfMissing<Type extends Pick<ICvResume, 'id'>>(
    cvResumeCollection: Type[],
    ...cvResumesToCheck: (Type | null | undefined)[]
  ): Type[] {
    const cvResumes: Type[] = cvResumesToCheck.filter(isPresent);
    if (cvResumes.length > 0) {
      const cvResumeCollectionIdentifiers = cvResumeCollection.map(cvResumeItem => this.getCvResumeIdentifier(cvResumeItem));
      const cvResumesToAdd = cvResumes.filter(cvResumeItem => {
        const cvResumeIdentifier = this.getCvResumeIdentifier(cvResumeItem);
        if (cvResumeCollectionIdentifiers.includes(cvResumeIdentifier)) {
          return false;
        }
        cvResumeCollectionIdentifiers.push(cvResumeIdentifier);
        return true;
      });
      return [...cvResumesToAdd, ...cvResumeCollection];
    }
    return cvResumeCollection;
  }

  protected convertValueFromClient<T extends ICvResume | NewCvResume | PartialUpdateCvResume>(cvResume: T): RestOf<T> {
    return {
      ...cvResume,
      createdAt: cvResume.createdAt?.toJSON() ?? null,
      updatedAt: cvResume.updatedAt?.toJSON() ?? null,
    };
  }

  protected convertResponseFromServer(res: RestCvResume): ICvResume {
    return this.convertValueFromServer(res);
  }

  protected convertResponseArrayFromServer(res: RestCvResume[]): ICvResume[] {
    return res.map(item => this.convertValueFromServer(item));
  }
}
