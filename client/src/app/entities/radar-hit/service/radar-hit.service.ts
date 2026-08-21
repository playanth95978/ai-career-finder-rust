import { HttpClient, HttpResponse, httpResource } from '@angular/common/http';
import { Injectable, computed, inject, signal } from '@angular/core';

import dayjs from 'dayjs/esm';
import { Observable, map } from 'rxjs';

import { ApplicationConfigService } from 'app/core/config/application-config.service';
import { createRequestOption } from 'app/core/request/request-util';
import { isPresent } from 'app/core/util/operators';
import { IRadarHit, NewRadarHit } from '../radar-hit.model';

export type PartialUpdateRadarHit = Partial<IRadarHit> & Pick<IRadarHit, 'id'>;

type RestOf<T extends IRadarHit | NewRadarHit> = Omit<T, 'createdAt'> & {
  createdAt?: string | null;
};

export type RestRadarHit = RestOf<IRadarHit>;

export type NewRestRadarHit = RestOf<NewRadarHit>;

export type PartialUpdateRestRadarHit = RestOf<PartialUpdateRadarHit>;

@Injectable()
export class RadarHitsService {
  readonly radarHitsParams = signal<Record<string, string | number | boolean | readonly (string | number | boolean)[]> | undefined>(
    undefined,
  );
  readonly radarHitsResource = httpResource<RestRadarHit[]>(() => {
    const params = this.radarHitsParams();
    if (!params) {
      return undefined;
    }
    return { url: this.resourceUrl, params };
  });
  /**
   * This signal holds the list of radarHit that have been fetched. It is updated when the radarHitsResource emits a new value.
   * In case of error while fetching the radarHits, the signal is set to an empty array.
   */
  readonly radarHits = computed(() =>
    (this.radarHitsResource.hasValue() ? this.radarHitsResource.value() : []).map(item => this.convertValueFromServer(item)),
  );
  protected readonly applicationConfigService = inject(ApplicationConfigService);
  protected readonly resourceUrl = this.applicationConfigService.getEndpointFor('api/radar-hits');

  protected convertValueFromServer(restRadarHit: RestRadarHit): IRadarHit {
    return {
      ...restRadarHit,
      createdAt: restRadarHit.createdAt ? dayjs(restRadarHit.createdAt) : undefined,
    };
  }
}

@Injectable({ providedIn: 'root' })
export class RadarHitService extends RadarHitsService {
  protected readonly http = inject(HttpClient);

  create(radarHit: NewRadarHit): Observable<IRadarHit> {
    const copy = this.convertValueFromClient(radarHit);
    return this.http.post<RestRadarHit>(this.resourceUrl, copy).pipe(map(res => this.convertResponseFromServer(res)));
  }

  update(radarHit: IRadarHit): Observable<IRadarHit> {
    const copy = this.convertValueFromClient(radarHit);
    return this.http
      .put<RestRadarHit>(`${this.resourceUrl}/${encodeURIComponent(this.getRadarHitIdentifier(radarHit))}`, copy)
      .pipe(map(res => this.convertResponseFromServer(res)));
  }

  partialUpdate(radarHit: PartialUpdateRadarHit): Observable<IRadarHit> {
    const copy = this.convertValueFromClient(radarHit);
    return this.http
      .patch<RestRadarHit>(`${this.resourceUrl}/${encodeURIComponent(this.getRadarHitIdentifier(radarHit))}`, copy)
      .pipe(map(res => this.convertResponseFromServer(res)));
  }

  find(id: number): Observable<IRadarHit> {
    return this.http
      .get<RestRadarHit>(`${this.resourceUrl}/${encodeURIComponent(id)}`)
      .pipe(map(res => this.convertResponseFromServer(res)));
  }

  query(req?: any): Observable<HttpResponse<IRadarHit[]>> {
    const options = createRequestOption(req);
    return this.http
      .get<RestRadarHit[]>(this.resourceUrl, { params: options, observe: 'response' })
      .pipe(map(res => res.clone({ body: this.convertResponseArrayFromServer(res.body!) })));
  }

  delete(id: number): Observable<undefined> {
    return this.http.delete<undefined>(`${this.resourceUrl}/${encodeURIComponent(id)}`);
  }

  getRadarHitIdentifier(radarHit: Pick<IRadarHit, 'id'>): number {
    return radarHit.id;
  }

  compareRadarHit(o1: Pick<IRadarHit, 'id'> | null, o2: Pick<IRadarHit, 'id'> | null): boolean {
    return o1 && o2 ? this.getRadarHitIdentifier(o1) === this.getRadarHitIdentifier(o2) : o1 === o2;
  }

  addRadarHitToCollectionIfMissing<Type extends Pick<IRadarHit, 'id'>>(
    radarHitCollection: Type[],
    ...radarHitsToCheck: (Type | null | undefined)[]
  ): Type[] {
    const radarHits: Type[] = radarHitsToCheck.filter(isPresent);
    if (radarHits.length > 0) {
      const radarHitCollectionIdentifiers = radarHitCollection.map(radarHitItem => this.getRadarHitIdentifier(radarHitItem));
      const radarHitsToAdd = radarHits.filter(radarHitItem => {
        const radarHitIdentifier = this.getRadarHitIdentifier(radarHitItem);
        if (radarHitCollectionIdentifiers.includes(radarHitIdentifier)) {
          return false;
        }
        radarHitCollectionIdentifiers.push(radarHitIdentifier);
        return true;
      });
      return [...radarHitsToAdd, ...radarHitCollection];
    }
    return radarHitCollection;
  }

  protected convertValueFromClient<T extends IRadarHit | NewRadarHit | PartialUpdateRadarHit>(radarHit: T): RestOf<T> {
    return {
      ...radarHit,
      createdAt: radarHit.createdAt?.toJSON() ?? null,
    };
  }

  protected convertResponseFromServer(res: RestRadarHit): IRadarHit {
    return this.convertValueFromServer(res);
  }

  protected convertResponseArrayFromServer(res: RestRadarHit[]): IRadarHit[] {
    return res.map(item => this.convertValueFromServer(item));
  }
}
