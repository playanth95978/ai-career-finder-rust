import { HttpClient, HttpResponse, httpResource } from '@angular/common/http';
import { Injectable, computed, inject, signal } from '@angular/core';

import dayjs from 'dayjs/esm';
import { Observable, map } from 'rxjs';

import { ApplicationConfigService } from 'app/core/config/application-config.service';
import { createRequestOption } from 'app/core/request/request-util';
import { isPresent } from 'app/core/util/operators';
import { IOfferTailoredResume, NewOfferTailoredResume } from '../offer-tailored-resume.model';

export type PartialUpdateOfferTailoredResume = Partial<IOfferTailoredResume> & Pick<IOfferTailoredResume, 'id'>;

type RestOf<T extends IOfferTailoredResume | NewOfferTailoredResume> = Omit<T, 'createdAt'> & {
  createdAt?: string | null;
};

export type RestOfferTailoredResume = RestOf<IOfferTailoredResume>;

export type NewRestOfferTailoredResume = RestOf<NewOfferTailoredResume>;

export type PartialUpdateRestOfferTailoredResume = RestOf<PartialUpdateOfferTailoredResume>;

@Injectable()
export class OfferTailoredResumesService {
  readonly offerTailoredResumesParams = signal<
    Record<string, string | number | boolean | readonly (string | number | boolean)[]> | undefined
  >(undefined);
  readonly offerTailoredResumesResource = httpResource<RestOfferTailoredResume[]>(() => {
    const params = this.offerTailoredResumesParams();
    if (!params) {
      return undefined;
    }
    return { url: this.resourceUrl, params };
  });
  /**
   * This signal holds the list of offerTailoredResume that have been fetched. It is updated when the offerTailoredResumesResource emits a new value.
   * In case of error while fetching the offerTailoredResumes, the signal is set to an empty array.
   */
  readonly offerTailoredResumes = computed(() =>
    (this.offerTailoredResumesResource.hasValue() ? this.offerTailoredResumesResource.value() : []).map(item =>
      this.convertValueFromServer(item),
    ),
  );
  protected readonly applicationConfigService = inject(ApplicationConfigService);
  protected readonly resourceUrl = this.applicationConfigService.getEndpointFor('api/offer-tailored-resumes');

  protected convertValueFromServer(restOfferTailoredResume: RestOfferTailoredResume): IOfferTailoredResume {
    return {
      ...restOfferTailoredResume,
      createdAt: restOfferTailoredResume.createdAt ? dayjs(restOfferTailoredResume.createdAt) : undefined,
    };
  }
}

@Injectable({ providedIn: 'root' })
export class OfferTailoredResumeService extends OfferTailoredResumesService {
  protected readonly http = inject(HttpClient);

  create(offerTailoredResume: NewOfferTailoredResume): Observable<IOfferTailoredResume> {
    const copy = this.convertValueFromClient(offerTailoredResume);
    return this.http.post<RestOfferTailoredResume>(this.resourceUrl, copy).pipe(map(res => this.convertResponseFromServer(res)));
  }

  update(offerTailoredResume: IOfferTailoredResume): Observable<IOfferTailoredResume> {
    const copy = this.convertValueFromClient(offerTailoredResume);
    return this.http
      .put<RestOfferTailoredResume>(
        `${this.resourceUrl}/${encodeURIComponent(this.getOfferTailoredResumeIdentifier(offerTailoredResume))}`,
        copy,
      )
      .pipe(map(res => this.convertResponseFromServer(res)));
  }

  partialUpdate(offerTailoredResume: PartialUpdateOfferTailoredResume): Observable<IOfferTailoredResume> {
    const copy = this.convertValueFromClient(offerTailoredResume);
    return this.http
      .patch<RestOfferTailoredResume>(
        `${this.resourceUrl}/${encodeURIComponent(this.getOfferTailoredResumeIdentifier(offerTailoredResume))}`,
        copy,
      )
      .pipe(map(res => this.convertResponseFromServer(res)));
  }

  find(id: number): Observable<IOfferTailoredResume> {
    return this.http
      .get<RestOfferTailoredResume>(`${this.resourceUrl}/${encodeURIComponent(id)}`)
      .pipe(map(res => this.convertResponseFromServer(res)));
  }

  query(req?: any): Observable<HttpResponse<IOfferTailoredResume[]>> {
    const options = createRequestOption(req);
    return this.http
      .get<RestOfferTailoredResume[]>(this.resourceUrl, { params: options, observe: 'response' })
      .pipe(map(res => res.clone({ body: this.convertResponseArrayFromServer(res.body!) })));
  }

  delete(id: number): Observable<undefined> {
    return this.http.delete<undefined>(`${this.resourceUrl}/${encodeURIComponent(id)}`);
  }

  getOfferTailoredResumeIdentifier(offerTailoredResume: Pick<IOfferTailoredResume, 'id'>): number {
    return offerTailoredResume.id;
  }

  compareOfferTailoredResume(o1: Pick<IOfferTailoredResume, 'id'> | null, o2: Pick<IOfferTailoredResume, 'id'> | null): boolean {
    return o1 && o2 ? this.getOfferTailoredResumeIdentifier(o1) === this.getOfferTailoredResumeIdentifier(o2) : o1 === o2;
  }

  addOfferTailoredResumeToCollectionIfMissing<Type extends Pick<IOfferTailoredResume, 'id'>>(
    offerTailoredResumeCollection: Type[],
    ...offerTailoredResumesToCheck: (Type | null | undefined)[]
  ): Type[] {
    const offerTailoredResumes: Type[] = offerTailoredResumesToCheck.filter(isPresent);
    if (offerTailoredResumes.length > 0) {
      const offerTailoredResumeCollectionIdentifiers = offerTailoredResumeCollection.map(offerTailoredResumeItem =>
        this.getOfferTailoredResumeIdentifier(offerTailoredResumeItem),
      );
      const offerTailoredResumesToAdd = offerTailoredResumes.filter(offerTailoredResumeItem => {
        const offerTailoredResumeIdentifier = this.getOfferTailoredResumeIdentifier(offerTailoredResumeItem);
        if (offerTailoredResumeCollectionIdentifiers.includes(offerTailoredResumeIdentifier)) {
          return false;
        }
        offerTailoredResumeCollectionIdentifiers.push(offerTailoredResumeIdentifier);
        return true;
      });
      return [...offerTailoredResumesToAdd, ...offerTailoredResumeCollection];
    }
    return offerTailoredResumeCollection;
  }

  protected convertValueFromClient<T extends IOfferTailoredResume | NewOfferTailoredResume | PartialUpdateOfferTailoredResume>(
    offerTailoredResume: T,
  ): RestOf<T> {
    return {
      ...offerTailoredResume,
      createdAt: offerTailoredResume.createdAt?.toJSON() ?? null,
    };
  }

  protected convertResponseFromServer(res: RestOfferTailoredResume): IOfferTailoredResume {
    return this.convertValueFromServer(res);
  }

  protected convertResponseArrayFromServer(res: RestOfferTailoredResume[]): IOfferTailoredResume[] {
    return res.map(item => this.convertValueFromServer(item));
  }
}
