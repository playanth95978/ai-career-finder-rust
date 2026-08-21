import { HttpClient, HttpResponse, httpResource } from '@angular/common/http';
import { Injectable, computed, inject, signal } from '@angular/core';

import dayjs from 'dayjs/esm';
import { Observable, map } from 'rxjs';

import { ApplicationConfigService } from 'app/core/config/application-config.service';
import { createRequestOption } from 'app/core/request/request-util';
import { isPresent } from 'app/core/util/operators';
import { IOfferPositioning, NewOfferPositioning } from '../offer-positioning.model';

export type PartialUpdateOfferPositioning = Partial<IOfferPositioning> & Pick<IOfferPositioning, 'id'>;

type RestOf<T extends IOfferPositioning | NewOfferPositioning> = Omit<T, 'createdAt'> & {
  createdAt?: string | null;
};

export type RestOfferPositioning = RestOf<IOfferPositioning>;

export type NewRestOfferPositioning = RestOf<NewOfferPositioning>;

export type PartialUpdateRestOfferPositioning = RestOf<PartialUpdateOfferPositioning>;

@Injectable()
export class OfferPositioningsService {
  readonly offerPositioningsParams = signal<Record<string, string | number | boolean | readonly (string | number | boolean)[]> | undefined>(
    undefined,
  );
  readonly offerPositioningsResource = httpResource<RestOfferPositioning[]>(() => {
    const params = this.offerPositioningsParams();
    if (!params) {
      return undefined;
    }
    return { url: this.resourceUrl, params };
  });
  /**
   * This signal holds the list of offerPositioning that have been fetched. It is updated when the offerPositioningsResource emits a new value.
   * In case of error while fetching the offerPositionings, the signal is set to an empty array.
   */
  readonly offerPositionings = computed(() =>
    (this.offerPositioningsResource.hasValue() ? this.offerPositioningsResource.value() : []).map(item =>
      this.convertValueFromServer(item),
    ),
  );
  protected readonly applicationConfigService = inject(ApplicationConfigService);
  protected readonly resourceUrl = this.applicationConfigService.getEndpointFor('api/offer-positionings');

  protected convertValueFromServer(restOfferPositioning: RestOfferPositioning): IOfferPositioning {
    return {
      ...restOfferPositioning,
      createdAt: restOfferPositioning.createdAt ? dayjs(restOfferPositioning.createdAt) : undefined,
    };
  }
}

@Injectable({ providedIn: 'root' })
export class OfferPositioningService extends OfferPositioningsService {
  protected readonly http = inject(HttpClient);

  create(offerPositioning: NewOfferPositioning): Observable<IOfferPositioning> {
    const copy = this.convertValueFromClient(offerPositioning);
    return this.http.post<RestOfferPositioning>(this.resourceUrl, copy).pipe(map(res => this.convertResponseFromServer(res)));
  }

  update(offerPositioning: IOfferPositioning): Observable<IOfferPositioning> {
    const copy = this.convertValueFromClient(offerPositioning);
    return this.http
      .put<RestOfferPositioning>(`${this.resourceUrl}/${encodeURIComponent(this.getOfferPositioningIdentifier(offerPositioning))}`, copy)
      .pipe(map(res => this.convertResponseFromServer(res)));
  }

  partialUpdate(offerPositioning: PartialUpdateOfferPositioning): Observable<IOfferPositioning> {
    const copy = this.convertValueFromClient(offerPositioning);
    return this.http
      .patch<RestOfferPositioning>(`${this.resourceUrl}/${encodeURIComponent(this.getOfferPositioningIdentifier(offerPositioning))}`, copy)
      .pipe(map(res => this.convertResponseFromServer(res)));
  }

  find(id: number): Observable<IOfferPositioning> {
    return this.http
      .get<RestOfferPositioning>(`${this.resourceUrl}/${encodeURIComponent(id)}`)
      .pipe(map(res => this.convertResponseFromServer(res)));
  }

  query(req?: any): Observable<HttpResponse<IOfferPositioning[]>> {
    const options = createRequestOption(req);
    return this.http
      .get<RestOfferPositioning[]>(this.resourceUrl, { params: options, observe: 'response' })
      .pipe(map(res => res.clone({ body: this.convertResponseArrayFromServer(res.body!) })));
  }

  delete(id: number): Observable<undefined> {
    return this.http.delete<undefined>(`${this.resourceUrl}/${encodeURIComponent(id)}`);
  }

  getOfferPositioningIdentifier(offerPositioning: Pick<IOfferPositioning, 'id'>): number {
    return offerPositioning.id;
  }

  compareOfferPositioning(o1: Pick<IOfferPositioning, 'id'> | null, o2: Pick<IOfferPositioning, 'id'> | null): boolean {
    return o1 && o2 ? this.getOfferPositioningIdentifier(o1) === this.getOfferPositioningIdentifier(o2) : o1 === o2;
  }

  addOfferPositioningToCollectionIfMissing<Type extends Pick<IOfferPositioning, 'id'>>(
    offerPositioningCollection: Type[],
    ...offerPositioningsToCheck: (Type | null | undefined)[]
  ): Type[] {
    const offerPositionings: Type[] = offerPositioningsToCheck.filter(isPresent);
    if (offerPositionings.length > 0) {
      const offerPositioningCollectionIdentifiers = offerPositioningCollection.map(offerPositioningItem =>
        this.getOfferPositioningIdentifier(offerPositioningItem),
      );
      const offerPositioningsToAdd = offerPositionings.filter(offerPositioningItem => {
        const offerPositioningIdentifier = this.getOfferPositioningIdentifier(offerPositioningItem);
        if (offerPositioningCollectionIdentifiers.includes(offerPositioningIdentifier)) {
          return false;
        }
        offerPositioningCollectionIdentifiers.push(offerPositioningIdentifier);
        return true;
      });
      return [...offerPositioningsToAdd, ...offerPositioningCollection];
    }
    return offerPositioningCollection;
  }

  protected convertValueFromClient<T extends IOfferPositioning | NewOfferPositioning | PartialUpdateOfferPositioning>(
    offerPositioning: T,
  ): RestOf<T> {
    return {
      ...offerPositioning,
      createdAt: offerPositioning.createdAt?.toJSON() ?? null,
    };
  }

  protected convertResponseFromServer(res: RestOfferPositioning): IOfferPositioning {
    return this.convertValueFromServer(res);
  }

  protected convertResponseArrayFromServer(res: RestOfferPositioning[]): IOfferPositioning[] {
    return res.map(item => this.convertValueFromServer(item));
  }
}
