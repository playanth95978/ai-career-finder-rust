import { HttpClient, HttpResponse, httpResource } from '@angular/common/http';
import { Injectable, computed, inject, signal } from '@angular/core';

import dayjs from 'dayjs/esm';
import { Observable, map } from 'rxjs';

import { ApplicationConfigService } from 'app/core/config/application-config.service';
import { createRequestOption } from 'app/core/request/request-util';
import { isPresent } from 'app/core/util/operators';
import { IRadarState, NewRadarState } from '../radar-state.model';

export type PartialUpdateRadarState = Partial<IRadarState> & Pick<IRadarState, 'id'>;

type RestOf<T extends IRadarState | NewRadarState> = Omit<T, 'lastOfferAt'> & {
  lastOfferAt?: string | null;
};

export type RestRadarState = RestOf<IRadarState>;

export type NewRestRadarState = RestOf<NewRadarState>;

export type PartialUpdateRestRadarState = RestOf<PartialUpdateRadarState>;

@Injectable()
export class RadarStatesService {
  readonly radarStatesParams = signal<Record<string, string | number | boolean | readonly (string | number | boolean)[]> | undefined>(
    undefined,
  );
  readonly radarStatesResource = httpResource<RestRadarState[]>(() => {
    const params = this.radarStatesParams();
    if (!params) {
      return undefined;
    }
    return { url: this.resourceUrl, params };
  });
  /**
   * This signal holds the list of radarState that have been fetched. It is updated when the radarStatesResource emits a new value.
   * In case of error while fetching the radarStates, the signal is set to an empty array.
   */
  readonly radarStates = computed(() =>
    (this.radarStatesResource.hasValue() ? this.radarStatesResource.value() : []).map(item => this.convertValueFromServer(item)),
  );
  protected readonly applicationConfigService = inject(ApplicationConfigService);
  protected readonly resourceUrl = this.applicationConfigService.getEndpointFor('api/radar-states');

  protected convertValueFromServer(restRadarState: RestRadarState): IRadarState {
    return {
      ...restRadarState,
      lastOfferAt: restRadarState.lastOfferAt ? dayjs(restRadarState.lastOfferAt) : undefined,
    };
  }
}

@Injectable({ providedIn: 'root' })
export class RadarStateService extends RadarStatesService {
  protected readonly http = inject(HttpClient);

  create(radarState: NewRadarState): Observable<IRadarState> {
    const copy = this.convertValueFromClient(radarState);
    return this.http.post<RestRadarState>(this.resourceUrl, copy).pipe(map(res => this.convertResponseFromServer(res)));
  }

  update(radarState: IRadarState): Observable<IRadarState> {
    const copy = this.convertValueFromClient(radarState);
    return this.http
      .put<RestRadarState>(`${this.resourceUrl}/${encodeURIComponent(this.getRadarStateIdentifier(radarState))}`, copy)
      .pipe(map(res => this.convertResponseFromServer(res)));
  }

  partialUpdate(radarState: PartialUpdateRadarState): Observable<IRadarState> {
    const copy = this.convertValueFromClient(radarState);
    return this.http
      .patch<RestRadarState>(`${this.resourceUrl}/${encodeURIComponent(this.getRadarStateIdentifier(radarState))}`, copy)
      .pipe(map(res => this.convertResponseFromServer(res)));
  }

  find(id: number): Observable<IRadarState> {
    return this.http
      .get<RestRadarState>(`${this.resourceUrl}/${encodeURIComponent(id)}`)
      .pipe(map(res => this.convertResponseFromServer(res)));
  }

  query(req?: any): Observable<HttpResponse<IRadarState[]>> {
    const options = createRequestOption(req);
    return this.http
      .get<RestRadarState[]>(this.resourceUrl, { params: options, observe: 'response' })
      .pipe(map(res => res.clone({ body: this.convertResponseArrayFromServer(res.body!) })));
  }

  delete(id: number): Observable<undefined> {
    return this.http.delete<undefined>(`${this.resourceUrl}/${encodeURIComponent(id)}`);
  }

  getRadarStateIdentifier(radarState: Pick<IRadarState, 'id'>): number {
    return radarState.id;
  }

  compareRadarState(o1: Pick<IRadarState, 'id'> | null, o2: Pick<IRadarState, 'id'> | null): boolean {
    return o1 && o2 ? this.getRadarStateIdentifier(o1) === this.getRadarStateIdentifier(o2) : o1 === o2;
  }

  addRadarStateToCollectionIfMissing<Type extends Pick<IRadarState, 'id'>>(
    radarStateCollection: Type[],
    ...radarStatesToCheck: (Type | null | undefined)[]
  ): Type[] {
    const radarStates: Type[] = radarStatesToCheck.filter(isPresent);
    if (radarStates.length > 0) {
      const radarStateCollectionIdentifiers = radarStateCollection.map(radarStateItem => this.getRadarStateIdentifier(radarStateItem));
      const radarStatesToAdd = radarStates.filter(radarStateItem => {
        const radarStateIdentifier = this.getRadarStateIdentifier(radarStateItem);
        if (radarStateCollectionIdentifiers.includes(radarStateIdentifier)) {
          return false;
        }
        radarStateCollectionIdentifiers.push(radarStateIdentifier);
        return true;
      });
      return [...radarStatesToAdd, ...radarStateCollection];
    }
    return radarStateCollection;
  }

  protected convertValueFromClient<T extends IRadarState | NewRadarState | PartialUpdateRadarState>(radarState: T): RestOf<T> {
    return {
      ...radarState,
      lastOfferAt: radarState.lastOfferAt?.toJSON() ?? null,
    };
  }

  protected convertResponseFromServer(res: RestRadarState): IRadarState {
    return this.convertValueFromServer(res);
  }

  protected convertResponseArrayFromServer(res: RestRadarState[]): IRadarState[] {
    return res.map(item => this.convertValueFromServer(item));
  }
}
