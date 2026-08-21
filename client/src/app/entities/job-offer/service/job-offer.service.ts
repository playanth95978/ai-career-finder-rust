import { HttpClient, HttpResponse, httpResource } from '@angular/common/http';
import { Injectable, computed, inject, signal } from '@angular/core';

import dayjs from 'dayjs/esm';
import { Observable, map } from 'rxjs';

import { ApplicationConfigService } from 'app/core/config/application-config.service';
import { createRequestOption } from 'app/core/request/request-util';
import { isPresent } from 'app/core/util/operators';
import { IJobOffer, NewJobOffer } from '../job-offer.model';

export type PartialUpdateJobOffer = Partial<IJobOffer> & Pick<IJobOffer, 'id'>;

type RestOf<T extends IJobOffer | NewJobOffer> = Omit<
  T,
  'publishedAt' | 'createdAt' | 'indexedAt' | 'updatedAt' | 'expiresAt' | 'lastCheckedAt'
> & {
  publishedAt?: string | null;
  createdAt?: string | null;
  indexedAt?: string | null;
  updatedAt?: string | null;
  expiresAt?: string | null;
  lastCheckedAt?: string | null;
};

export type RestJobOffer = RestOf<IJobOffer>;

export type NewRestJobOffer = RestOf<NewJobOffer>;

export type PartialUpdateRestJobOffer = RestOf<PartialUpdateJobOffer>;

@Injectable()
export class JobOffersService {
  readonly jobOffersParams = signal<Record<string, string | number | boolean | readonly (string | number | boolean)[]> | undefined>(
    undefined,
  );
  readonly jobOffersResource = httpResource<RestJobOffer[]>(() => {
    const params = this.jobOffersParams();
    if (!params) {
      return undefined;
    }
    return { url: this.resourceUrl, params };
  });
  /**
   * This signal holds the list of jobOffer that have been fetched. It is updated when the jobOffersResource emits a new value.
   * In case of error while fetching the jobOffers, the signal is set to an empty array.
   */
  readonly jobOffers = computed(() =>
    (this.jobOffersResource.hasValue() ? this.jobOffersResource.value() : []).map(item => this.convertValueFromServer(item)),
  );
  protected readonly applicationConfigService = inject(ApplicationConfigService);
  protected readonly resourceUrl = this.applicationConfigService.getEndpointFor('api/job-offers');

  protected convertValueFromServer(restJobOffer: RestJobOffer): IJobOffer {
    return {
      ...restJobOffer,
      publishedAt: restJobOffer.publishedAt ? dayjs(restJobOffer.publishedAt) : undefined,
      createdAt: restJobOffer.createdAt ? dayjs(restJobOffer.createdAt) : undefined,
      indexedAt: restJobOffer.indexedAt ? dayjs(restJobOffer.indexedAt) : undefined,
      updatedAt: restJobOffer.updatedAt ? dayjs(restJobOffer.updatedAt) : undefined,
      expiresAt: restJobOffer.expiresAt ? dayjs(restJobOffer.expiresAt) : undefined,
      lastCheckedAt: restJobOffer.lastCheckedAt ? dayjs(restJobOffer.lastCheckedAt) : undefined,
    };
  }
}

@Injectable({ providedIn: 'root' })
export class JobOfferService extends JobOffersService {
  protected readonly http = inject(HttpClient);

  create(jobOffer: NewJobOffer): Observable<IJobOffer> {
    const copy = this.convertValueFromClient(jobOffer);
    return this.http.post<RestJobOffer>(this.resourceUrl, copy).pipe(map(res => this.convertResponseFromServer(res)));
  }

  update(jobOffer: IJobOffer): Observable<IJobOffer> {
    const copy = this.convertValueFromClient(jobOffer);
    return this.http
      .put<RestJobOffer>(`${this.resourceUrl}/${encodeURIComponent(this.getJobOfferIdentifier(jobOffer))}`, copy)
      .pipe(map(res => this.convertResponseFromServer(res)));
  }

  partialUpdate(jobOffer: PartialUpdateJobOffer): Observable<IJobOffer> {
    const copy = this.convertValueFromClient(jobOffer);
    return this.http
      .patch<RestJobOffer>(`${this.resourceUrl}/${encodeURIComponent(this.getJobOfferIdentifier(jobOffer))}`, copy)
      .pipe(map(res => this.convertResponseFromServer(res)));
  }

  find(id: number): Observable<IJobOffer> {
    return this.http
      .get<RestJobOffer>(`${this.resourceUrl}/${encodeURIComponent(id)}`)
      .pipe(map(res => this.convertResponseFromServer(res)));
  }

  query(req?: any): Observable<HttpResponse<IJobOffer[]>> {
    const options = createRequestOption(req);
    return this.http
      .get<RestJobOffer[]>(this.resourceUrl, { params: options, observe: 'response' })
      .pipe(map(res => res.clone({ body: this.convertResponseArrayFromServer(res.body!) })));
  }

  delete(id: number): Observable<undefined> {
    return this.http.delete<undefined>(`${this.resourceUrl}/${encodeURIComponent(id)}`);
  }

  getJobOfferIdentifier(jobOffer: Pick<IJobOffer, 'id'>): number {
    return jobOffer.id;
  }

  compareJobOffer(o1: Pick<IJobOffer, 'id'> | null, o2: Pick<IJobOffer, 'id'> | null): boolean {
    return o1 && o2 ? this.getJobOfferIdentifier(o1) === this.getJobOfferIdentifier(o2) : o1 === o2;
  }

  addJobOfferToCollectionIfMissing<Type extends Pick<IJobOffer, 'id'>>(
    jobOfferCollection: Type[],
    ...jobOffersToCheck: (Type | null | undefined)[]
  ): Type[] {
    const jobOffers: Type[] = jobOffersToCheck.filter(isPresent);
    if (jobOffers.length > 0) {
      const jobOfferCollectionIdentifiers = jobOfferCollection.map(jobOfferItem => this.getJobOfferIdentifier(jobOfferItem));
      const jobOffersToAdd = jobOffers.filter(jobOfferItem => {
        const jobOfferIdentifier = this.getJobOfferIdentifier(jobOfferItem);
        if (jobOfferCollectionIdentifiers.includes(jobOfferIdentifier)) {
          return false;
        }
        jobOfferCollectionIdentifiers.push(jobOfferIdentifier);
        return true;
      });
      return [...jobOffersToAdd, ...jobOfferCollection];
    }
    return jobOfferCollection;
  }

  protected convertValueFromClient<T extends IJobOffer | NewJobOffer | PartialUpdateJobOffer>(jobOffer: T): RestOf<T> {
    return {
      ...jobOffer,
      publishedAt: jobOffer.publishedAt?.toJSON() ?? null,
      createdAt: jobOffer.createdAt?.toJSON() ?? null,
      indexedAt: jobOffer.indexedAt?.toJSON() ?? null,
      updatedAt: jobOffer.updatedAt?.toJSON() ?? null,
      expiresAt: jobOffer.expiresAt?.toJSON() ?? null,
      lastCheckedAt: jobOffer.lastCheckedAt?.toJSON() ?? null,
    };
  }

  protected convertResponseFromServer(res: RestJobOffer): IJobOffer {
    return this.convertValueFromServer(res);
  }

  protected convertResponseArrayFromServer(res: RestJobOffer[]): IJobOffer[] {
    return res.map(item => this.convertValueFromServer(item));
  }
}
