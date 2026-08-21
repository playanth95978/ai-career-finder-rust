import { HttpClient, HttpResponse, httpResource } from '@angular/common/http';
import { Injectable, computed, inject, signal } from '@angular/core';

import { Observable } from 'rxjs';

import { ApplicationConfigService } from 'app/core/config/application-config.service';
import { createRequestOption } from 'app/core/request/request-util';
import { isPresent } from 'app/core/util/operators';
import { IUserPreference, NewUserPreference } from '../user-preference.model';

export type PartialUpdateUserPreference = Partial<IUserPreference> & Pick<IUserPreference, 'id'>;

@Injectable()
export class UserPreferencesService {
  readonly userPreferencesParams = signal<Record<string, string | number | boolean | readonly (string | number | boolean)[]> | undefined>(
    undefined,
  );
  readonly userPreferencesResource = httpResource<IUserPreference[]>(() => {
    const params = this.userPreferencesParams();
    if (!params) {
      return undefined;
    }
    return { url: this.resourceUrl, params };
  });
  /**
   * This signal holds the list of userPreference that have been fetched. It is updated when the userPreferencesResource emits a new value.
   * In case of error while fetching the userPreferences, the signal is set to an empty array.
   */
  readonly userPreferences = computed(() => (this.userPreferencesResource.hasValue() ? this.userPreferencesResource.value() : []));
  protected readonly applicationConfigService = inject(ApplicationConfigService);
  protected readonly resourceUrl = this.applicationConfigService.getEndpointFor('api/user-preferences');
}

@Injectable({ providedIn: 'root' })
export class UserPreferenceService extends UserPreferencesService {
  protected readonly http = inject(HttpClient);

  create(userPreference: NewUserPreference): Observable<IUserPreference> {
    return this.http.post<IUserPreference>(this.resourceUrl, userPreference);
  }

  update(userPreference: IUserPreference): Observable<IUserPreference> {
    return this.http.put<IUserPreference>(
      `${this.resourceUrl}/${encodeURIComponent(this.getUserPreferenceIdentifier(userPreference))}`,
      userPreference,
    );
  }

  partialUpdate(userPreference: PartialUpdateUserPreference): Observable<IUserPreference> {
    return this.http.patch<IUserPreference>(
      `${this.resourceUrl}/${encodeURIComponent(this.getUserPreferenceIdentifier(userPreference))}`,
      userPreference,
    );
  }

  find(id: number): Observable<IUserPreference> {
    return this.http.get<IUserPreference>(`${this.resourceUrl}/${encodeURIComponent(id)}`);
  }

  query(req?: any): Observable<HttpResponse<IUserPreference[]>> {
    const options = createRequestOption(req);
    return this.http.get<IUserPreference[]>(this.resourceUrl, { params: options, observe: 'response' });
  }

  delete(id: number): Observable<undefined> {
    return this.http.delete<undefined>(`${this.resourceUrl}/${encodeURIComponent(id)}`);
  }

  getUserPreferenceIdentifier(userPreference: Pick<IUserPreference, 'id'>): number {
    return userPreference.id;
  }

  compareUserPreference(o1: Pick<IUserPreference, 'id'> | null, o2: Pick<IUserPreference, 'id'> | null): boolean {
    return o1 && o2 ? this.getUserPreferenceIdentifier(o1) === this.getUserPreferenceIdentifier(o2) : o1 === o2;
  }

  addUserPreferenceToCollectionIfMissing<Type extends Pick<IUserPreference, 'id'>>(
    userPreferenceCollection: Type[],
    ...userPreferencesToCheck: (Type | null | undefined)[]
  ): Type[] {
    const userPreferences: Type[] = userPreferencesToCheck.filter(isPresent);
    if (userPreferences.length > 0) {
      const userPreferenceCollectionIdentifiers = userPreferenceCollection.map(userPreferenceItem =>
        this.getUserPreferenceIdentifier(userPreferenceItem),
      );
      const userPreferencesToAdd = userPreferences.filter(userPreferenceItem => {
        const userPreferenceIdentifier = this.getUserPreferenceIdentifier(userPreferenceItem);
        if (userPreferenceCollectionIdentifiers.includes(userPreferenceIdentifier)) {
          return false;
        }
        userPreferenceCollectionIdentifiers.push(userPreferenceIdentifier);
        return true;
      });
      return [...userPreferencesToAdd, ...userPreferenceCollection];
    }
    return userPreferenceCollection;
  }
}
